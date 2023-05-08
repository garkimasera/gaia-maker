pub fn linear_interpolation(table: &[(f32, f32)], x: f32) -> f32 {
    assert!(table.len() > 2);
    let first = table.first().unwrap();
    let last = table.last().unwrap();
    if first.0 >= x {
        return first.1;
    } else if last.0 <= x {
        return last.1;
    }

    for i in 0..(table.len() - 1) {
        let x0 = table[i].0;
        let x1 = table[i + 1].0;
        if x0 < x && x <= x1 {
            let y0 = table[i].1;
            let y1 = table[i + 1].1;
            let a = (y1 - y0) / (x1 - x0);
            let b = (x1 * y0 - x0 * y1) / (x1 - x0);
            return a * x + b;
        }
    }

    panic!("invalid input for interpolation: {}", x)
}
