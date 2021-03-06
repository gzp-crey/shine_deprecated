use crate::geometry2::{CollinearTest, CollinearTestType, Orientation, OrientationType, Predicates};
use crate::triangulation::graph::{Face, PredicatesContext, Triangulation, Vertex};
use crate::triangulation::query::{TopologyQuery, VertexClue};
use crate::triangulation::traverse::EdgeCirculator;
use crate::triangulation::types::{FaceIndex, FaceVertex, Rot3, VertexIndex};
use std::mem;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Crossing {
    Start { face: FaceIndex, vertex: Rot3 },
    End { face: FaceIndex, vertex: Rot3 },
    CoincidentEdge { face: FaceIndex, edge: Rot3 },
    PositiveEdge { face: FaceIndex, edge: Rot3 },
    NegativeEdge { face: FaceIndex, edge: Rot3 },
}

pub struct CrossingIterator<'a, PR, V, F, C>
where
    PR: Predicates,
    V: Vertex<Position = PR::Position>,
    F: Face,
    C: PredicatesContext<Predicates = PR>,
{
    tri: &'a Triangulation<PR::Position, V, F, C>,
    v0: VertexIndex,
    v1: VertexIndex,
    current: Option<Crossing>,
}

impl<'a, PR, V, F, C> CrossingIterator<'a, PR, V, F, C>
where
    PR: Predicates,
    V: Vertex<Position = PR::Position>,
    F: Face,
    C: PredicatesContext<Predicates = PR>,
{
    pub fn new(
        tri: &Triangulation<PR::Position, V, F, C>,
        v0: VertexIndex,
        v1: VertexIndex,
    ) -> CrossingIterator<'_, PR, V, F, C> {
        assert_eq!(tri.dimension(), 2);
        assert_ne!(v0, v1);
        assert!(tri.is_finite_vertex(v0));
        assert!(tri.is_finite_vertex(v1));

        let mut iter = CrossingIterator {
            tri,
            v0,
            v1,
            current: None,
        };
        iter.current = iter.search_vertex(iter.v0, iter.v0);
        iter
    }

    fn advance(&mut self) -> Option<Crossing> {
        let next = match self.current {
            None => None,
            Some(Crossing::Start { face, vertex }) => self.search_edge(face, vertex),
            Some(Crossing::End { face, vertex }) => self.search_vertex(self.tri.vi(FaceVertex { face, vertex }), self.v0),
            Some(Crossing::CoincidentEdge { face, edge }) => self.search_vertex(
                self.tri.vi(VertexClue::edge_end(face, edge)),
                self.tri.vi(VertexClue::edge_start(face, edge)),
            ),
            Some(Crossing::PositiveEdge { face, edge }) => self.search_edge(face, edge),
            Some(Crossing::NegativeEdge { face, edge }) => self.search_edge(face, edge),
        };

        mem::replace(&mut self.current, next)
    }

    /// Find next crossing edge by circulating the edges around the base_vertex.
    /// start_vertex is used to avoid going backward whan collinear edges are detected.
    fn search_vertex(&self, base_vertex: VertexIndex, start_vertex: VertexIndex) -> Option<Crossing> {
        let mut start_orientation = OrientationType::Collinear;
        let mut circulator = EdgeCirculator::new(self.tri, base_vertex);

        if base_vertex == self.v1 {
            return None;
        }

        let pr = self.tri.context.predicates();

        loop {
            let vertex = circulator.end_vertex();
            if self.tri.is_infinite_vertex(vertex) || vertex == self.v0 {
                // skip infinite edges
                circulator.advance_cw();
                continue;
            }

            if vertex == self.v1 {
                return Some(Crossing::CoincidentEdge {
                    face: circulator.face(),
                    edge: circulator.edge(),
                });
            }

            let orientation = if vertex == start_vertex {
                // we are on the edge (base_vertex, start_vertex) edge which is just the opposite
                // direction of the crosiing edge, thus any orientation can be picked for the rotate
                OrientationType::CCW
            } else {
                let p0 = &self.tri.p(self.v0);
                let p1 = &self.tri.p(self.v1);
                let pos = &self.tri.p(vertex);

                let orient = pr.orientation_triangle(p0, p1, pos);
                if orient.is_collinear() {
                    let collinear_test = pr.test_collinear_points(p0, p1, pos).into_type();
                    match collinear_test {
                        CollinearTestType::Before => {
                            // it's an edge just in the other direction on collinear to the v0-v1 segment, select some "random" orientation
                            OrientationType::CCW
                        }
                        CollinearTestType::First => {
                            panic!("invalid triangulation, p0 == pos; p0 == edge.start; edge.end == p, edge has a zero length")
                        }
                        CollinearTestType::Between => {
                            // pe is between p0 and p1
                            return Some(Crossing::CoincidentEdge {
                                face: circulator.face(),
                                edge: circulator.edge(),
                            });
                        }
                        CollinearTestType::Second => {
                            panic!("invalid triangulation, p1 == pos, but v1 != vertex, distinct vertices with the same position")
                        }
                        CollinearTestType::After => {
                            panic!("invalid triangulation, collinear, pos is not contained in the (p0,p1) segment")
                        }
                    }
                } else if orient.is_ccw() {
                    OrientationType::CCW
                } else {
                    OrientationType::CW
                }
            };

            if start_orientation == OrientationType::Collinear {
                // "first" loop iteration, find circulating direction
                assert!(orientation == OrientationType::CW || orientation == OrientationType::CCW);
                start_orientation = orientation;
            }

            if start_orientation != orientation {
                // orientation has changed -> we have the edge crossing the query
                if orientation == OrientationType::CCW {
                    // we have just passed our edge, go back
                    circulator.advance_cw();
                }

                return Some(Crossing::Start {
                    face: circulator.face(),
                    vertex: circulator.edge().increment(),
                });
            } else if start_orientation == OrientationType::CCW {
                circulator.advance_cw();
            } else {
                assert_eq!(start_orientation, OrientationType::CW);
                circulator.advance_ccw();
            }
        }
    }

    /// Find next crossing edge by checking the opposite face.
    fn search_edge(&self, start_face: FaceIndex, start_edge: Rot3) -> Option<Crossing> {
        let face = self.tri[start_face].neighbor(start_edge);
        let vertex_index = self.tri[face].get_neighbor_index(start_face).unwrap();
        let vertex = self.tri[face].vertex(vertex_index);

        if vertex == self.v1 {
            return Some(Crossing::End {
                face,
                vertex: vertex_index,
            });
        };

        let p0 = &self.tri.p(self.v0);
        let p1 = &self.tri.p(self.v1);
        let pn = &self.tri.p(vertex);
        let pr = self.tri.context.predicates();

        let orientation = pr.orientation_triangle(p0, p1, pn);
        if orientation.is_collinear() {
            Some(Crossing::End {
                face,
                vertex: vertex_index,
            })
        } else if orientation.is_ccw() {
            Some(Crossing::NegativeEdge {
                face,
                edge: vertex_index.increment(),
            })
        } else {
            Some(Crossing::PositiveEdge {
                face,
                edge: vertex_index.decrement(),
            })
        }
    }
}

impl<'a, PR, V, F, C> Iterator for CrossingIterator<'a, PR, V, F, C>
where
    PR: Predicates,
    V: Vertex<Position = PR::Position>,
    F: Face,
    C: PredicatesContext<Predicates = PR>,
{
    type Item = Crossing;

    fn next(&mut self) -> Option<Self::Item> {
        self.advance()
    }
}
