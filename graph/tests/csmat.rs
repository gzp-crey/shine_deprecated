extern crate shine_graph;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate rand;

use rand::Rng;

use shine_graph::csmat::*;

fn csr_simple_<M: CSMat<Item = (usize, usize)>>(mut matrix: M) {
    for i in 0..2 {
        trace!("pass: {}", i);

        assert_eq!(matrix.nnz(), 0);

        assert_eq!(matrix.get(0, 0), None);
        matrix.add(0, 0, (0, 0));
        assert_eq!(matrix.get(0, 0), Some(&(0, 0)));
        assert_eq!(matrix.nnz(), 1);

        assert_eq!(matrix.get(3, 4), None);
        matrix.add(3, 4, (3, 4));
        assert_eq!(matrix.get(3, 4), Some(&(3, 4)));
        assert_eq!(matrix.nnz(), 2);

        assert_eq!(matrix.remove(3, 4), Some((3, 4)));
        assert_eq!(matrix.get(3, 4), None);
        assert_eq!(matrix.nnz(), 1);

        matrix.add(4, 3, (4, 3));
        assert_eq!(matrix.get(4, 3), Some(&(4, 3)));
        assert_eq!(matrix.nnz(), 2);

        matrix.add(4, 2, (4, 2));
        assert_eq!(matrix.get(4, 2), Some(&(4, 2)));
        assert_eq!(matrix.nnz(), 3);

        matrix.add(4, 4, (4, 4));
        assert_eq!(matrix.get(0, 0), Some(&(0, 0)));
        assert_eq!(matrix.get(4, 2), Some(&(4, 2)));
        assert_eq!(matrix.get(4, 3), Some(&(4, 3)));
        assert_eq!(matrix.get(4, 4), Some(&(4, 4)));
        assert_eq!(matrix.nnz(), 4);

        matrix.clear();
        assert_eq!(matrix.nnz(), 0);
        assert_eq!(matrix.get(0, 0), None);
        assert_eq!(matrix.get(4, 2), None);
        assert_eq!(matrix.get(4, 3), None);
        assert_eq!(matrix.get(4, 4), None);
    }
}

fn csr_stress_<M: CSMat<Item = (usize, usize)>>(mut matrix: M) {
    let mx = vec![vec![0; 128]];
    let cnt = 70;

    let mut rng = rand::thread_rng();
    for _ in 0..cnt {
        let r = rng.gen_range(0usize, 128);
        let c = rng.gen_range(0usize, 128);
        if mx[r][c] == 1 {
            assert_eq!(matrix.get(r, c), None);
            matrix.add(r, c, (r, c));
        } else {
            assert_eq!(matrix.get(r, c), Some(&(r, c)));
        }
    }

    for r in 0..128 {
        for c in 0..128 {
            if mx[r][c] == 0 {
                assert_eq!(matrix.get(r, c), None);
            } else {
                assert_eq!(matrix.get(r, c), Some(&(r, c)));
            }
        }
    }
}

#[test]
fn csr_simple() {
    let _ = env_logger::try_init();

    for _ in 0..100 {
        trace!("CSVecMat - row");
        csr_simple_(CSVecMat::<(usize, usize)>::new_row());
        csr_stress_(CSVecMat::<(usize, usize)>::new_row());

        trace!("CSVecMat - col");
        csr_simple_(CSVecMat::<(usize, usize)>::new_column());
        csr_stress_(CSVecMat::<(usize, usize)>::new_column());

        trace!("CSArenaMat - row");
        csr_simple_(CSArenaMat::<(usize, usize)>::new_row());
        csr_stress_(CSArenaMat::<(usize, usize)>::new_row());

        trace!("CSArenaMat - col");
        csr_simple_(CSArenaMat::<(usize, usize)>::new_column());
        csr_stress_(CSArenaMat::<(usize, usize)>::new_column());
    }
}
