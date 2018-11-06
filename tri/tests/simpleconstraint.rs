#![feature(custom_attribute)]

extern crate shine_testutils;
extern crate shine_tri;
#[macro_use]
extern crate log;

mod common;

use common::{Sample, SimpleConstraint, SimpleFace, SimpleTrif32, SimpleTrif64, SimpleTrii32, SimpleTrii64, SimpleVertex};
use shine_testutils::init_test;
use shine_tri::geometry::{Position, Predicates, Real};
use shine_tri::{Builder, Checker, Triangulation};

#[test]
fn constraint_segment() {
    init_test(module_path!());

    fn test_<R, P, PR>(mut tri: Triangulation<PR, SimpleVertex<P>, SimpleFace>, desc: &str)
    where
        R: Real,
        P: Default + Position<Real = R> + From<Sample>,
        PR: Default + Predicates<Position = P, Real = R>,
    {
        info!("{}", desc);

        let transforms: Vec<(&str, Box<Fn(f32) -> P>)> = vec![
            ("(x, 0)", Box::new(|x| Sample(x, 0.).into())),
            ("(0, x)", Box::new(|x| Sample(0., x).into())),
            ("(-x, 0)", Box::new(|x| Sample(-x, 0.).into())),
            ("(0, -x)", Box::new(|x| Sample(0., -x).into())),
            ("(x, x)", Box::new(|x| Sample(x, x).into())),
            ("(x, -x)", Box::new(|x| Sample(x, -x).into())),
            ("(-x, -x)", Box::new(|x| Sample(-x, -x).into())),
            ("(-x, x)", Box::new(|x| Sample(-x, x).into())),
        ];

        for (info, map) in transforms.iter() {
            debug!("transformation: {}", info);

            //fTriTrace.setVirtualPositions( { glm::vec2( -1.5f, 0.0f ), glm::vec2( 1.5f, 0.0f ), glm::vec2( 0.0f, 1.5f ), glm::vec2( 0.0f, -1.5f ) } );

            tri.add_vertex(map(0.), None);
            assert_eq!(tri.check(None), Ok(()));
            tri.add_vertex(map(1.), None);
            assert_eq!(tri.check(None), Ok(()));

            tri.add_constraint_segment(map(0.2), map(0.5), SimpleConstraint(1));
            assert_eq!(tri.check(None), Ok(()));
            tri.add_constraint_segment(map(0.3), map(0.7), SimpleConstraint(2));
            assert_eq!(tri.check(None), Ok(()));
            tri.add_constraint_segment(map(0.8), map(0.1), SimpleConstraint(4));
            assert_eq!(tri.check(None), Ok(()));

            trace!("clear");
            tri.graph.clear();
            assert!(tri.graph.is_empty());
            assert_eq!(tri.check(None), Ok(()));
        }
    }

    test_(SimpleTrif32::default(), "inexact f32");
    test_(SimpleTrif64::default(), "inexact f64");
    test_(SimpleTrii32::default(), "exact i32");
    test_(SimpleTrii64::default(), "exact i64");
}

#[test]
#[ignore]
fn constraint_simple1() {
    init_test(module_path!());

    fn test_<R, P, PR>(mut tri: Triangulation<PR, SimpleVertex<P>, SimpleFace>, desc: &str)
    where
        R: Real,
        P: Default + Position<Real = R> + From<Sample>,
        PR: Default + Predicates<Position = P, Real = R>,
    {
        info!("{}", desc);

       let transforms: Vec<(&str, Box<Fn(f32, f32) -> P>)> = vec![
            ("(x, y)", Box::new(|x, y| Sample(x, y).into())),
            ("(-x, y)", Box::new(|x, y| Sample(-x, y).into())),
            ("(-x, -y)", Box::new(|x, y| Sample(-x, -y).into())),
            ("(x, -y)", Box::new(|x, y| Sample(x, -y).into())),
            ("(y, x)", Box::new(|x, y| Sample(y, x).into())),
            ("(-y, x)", Box::new(|x, y| Sample(-y, x).into())),
            ("(-y, -x)", Box::new(|x, y| Sample(-y, -x).into())),
            ("(y, -x)", Box::new(|x, y| Sample(y, -x).into())),
        ];

        for (info, map) in transforms.iter() {
            debug!("transformation: {}", info);

            //fTriTrace.setVirtualPositions( { glm::vec2( -1.5f, 0.0f ), glm::vec2( 1.5f, 0.0f ), glm::vec2( 0.0f, 1.5f ), glm::vec2( 0.0f, -1.5f ) } );

            tri.add_vertex( map( 0., 0. ), None );
            assert_eq!(tri.check(None), Ok(()));
            tri.add_vertex( map( 1., 0. ), None );
            assert_eq!(tri.check(None), Ok(()));
            tri.add_vertex( map( 1., 1. ), None );
            assert_eq!(tri.check(None), Ok(()));

            tri.add_constraint_segment( map( 0., 0. ), map( 1., 0. ), SimpleConstraint(1) );
            assert_eq!(tri.check(None), Ok(()));
            tri.add_constraint_segment( map( 0., 0. ), map( 1., 1. ), SimpleConstraint(2) );
            assert_eq!(tri.check(None), Ok(()));
            tri.add_constraint_segment( map( 1., 0. ), map( 1., 1. ), SimpleConstraint(4) );
            assert_eq!(tri.check(None), Ok(()));

            trace!("clear");
            tri.graph.clear();
            assert!(tri.graph.is_empty());
            assert_eq!(tri.check(None), Ok(()));
        }
    }

    test_(SimpleTrif32::default(), "inexact f32");
    test_(SimpleTrif64::default(), "inexact f64");
    test_(SimpleTrii32::default(), "exact i32");
    test_(SimpleTrii64::default(), "exact i64");
}

#[test]
#[ignore]
fn constraint_simple2() {
    init_test(module_path!());

    fn test_<R, P, PR>(mut tri: Triangulation<PR, SimpleVertex<P>, SimpleFace>, desc: &str)
    where
        R: Real,
        P: Default + Position<Real = R> + From<Sample>,
        PR: Default + Predicates<Position = P, Real = R>,
    {
        info!("{}", desc);

       let transforms: Vec<(&str, Box<Fn(f32, f32) -> P>)> = vec![
            ("(x, y)", Box::new(|x, y| Sample(x, y).into())),
            ("(-x, y)", Box::new(|x, y| Sample(-x, y).into())),
            ("(-x, -y)", Box::new(|x, y| Sample(-x, -y).into())),
            ("(x, -y)", Box::new(|x, y| Sample(x, -y).into())),
            ("(y, x)", Box::new(|x, y| Sample(y, x).into())),
            ("(-y, x)", Box::new(|x, y| Sample(-y, x).into())),
            ("(-y, -x)", Box::new(|x, y| Sample(-y, -x).into())),
            ("(y, -x)", Box::new(|x, y| Sample(y, -x).into())),
        ];

        for (info, map) in transforms.iter() {
            debug!("transformation: {}", info);

            //fTriTrace.setVirtualPositions( { glm::vec2( -1.5f, 0.0f ), glm::vec2( 1.5f, 0.0f ), glm::vec2( 0.0f, 1.5f ), glm::vec2( 0.0f, -1.5f ) } );

           tri.add_vertex( map( 0., 0. ), None );
            tri.add_vertex( map( 1., 0. ), None );
            tri.add_vertex( map( 1., 1. ), None );

            tri.add_constraint_segment( map( 1., 0. ), map( 1., 1. ), SimpleConstraint(1) ); 
            assert_eq!(tri.check(None), Ok(()));
            tri.add_constraint_segment( map( 0.2, 0. ), map( 0.5, 0. ), SimpleConstraint(2) ); 
            assert_eq!(tri.check(None), Ok(()));
            tri.add_constraint_segment( map( 0.3, 0. ), map( 0.7, 0. ), SimpleConstraint(4) ); 
            assert_eq!(tri.check(None), Ok(()));
            tri.add_constraint_segment( map( 0., 0. ), map( 1., 0. ), SimpleConstraint(8) ); 
            assert_eq!(tri.check(None), Ok(()));
            tri.add_constraint_segment( map( 1., 0. ), map( 0., 0. ), SimpleConstraint(16) ); 
            assert_eq!(tri.check(None), Ok(()));
            tri.add_constraint_segment( map( 1., 1. ), map( 0., 0. ), SimpleConstraint(32) );
            assert_eq!(tri.check(None), Ok(()));
            tri.add_constraint_segment( map( 0.1, 0.1 ), map( 0.9, 0.9 ), SimpleConstraint(64) );
            assert_eq!(tri.check(None), Ok(()));
            tri.add_constraint_segment( map( 0.9, 0.9 ), map( 0.1, 0.1 ), SimpleConstraint(128) );
            assert_eq!(tri.check(None), Ok(()));
            tri.add_constraint_segment( map( 0.8, 0.8 ), map( 0.2, 0.2 ), SimpleConstraint(256) );
            assert_eq!(tri.check(None), Ok(()));

            tri.add_vertex( map( 0.2, 0.5 ), None ); 
            assert_eq!(tri.check(None), Ok(()));
            tri.add_vertex( map( 0.5, 0.2 ), None ); 
            assert_eq!(tri.check(None), Ok(()));
            tri.add_vertex( map( 0.5, 0.5 ), None ); 
            assert_eq!(tri.check(None), Ok(()));


            trace!("clear");
            tri.graph.clear();
            assert!(tri.graph.is_empty());
            assert_eq!(tri.check(None), Ok(()));
        }
    }

    test_(SimpleTrif32::default(), "inexact f32");
    test_(SimpleTrif64::default(), "inexact f64");
    test_(SimpleTrii32::default(), "exact i32");
    test_(SimpleTrii64::default(), "exact i64");
}
