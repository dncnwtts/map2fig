fn main() {
    // Test the half-open interval logic
    let p1 = (0i32, 100i32);  // tip
    let p2 = (9i32, 50i32);   // base top
    let y = 50i32;
    
    let (x1, y1) = p1;
    let (x2, y2) = p2;
    
    println!("Edge from {:?} to {:?}, checking y={}", p1, p2, y);
    println!("  y1={}, y2={}", y1, y2);
    println!("  Condition: (y > y1 && y <= y2) || (y > y2 && y <= y1)");
    println!("  = ({} > {} && {} <= {}) || ({} > {} && {} <= {})", y, y1, y, y2, y, y2, y, y1);
    println!("  = ({} && {}) || ({} && {})", y > y1, y <= y2, y > y2, y <= y1);
    
    let cond1 = y > y1 && y <= y2;
    let cond2 = y > y2 && y <= y1;
    println!("  = {} || {} = {}", cond1, cond2, cond1 || cond2);
    
    if cond1 || cond2 {
        println!("  => FOUND");
        let t = (y as f64 - y1 as f64) / (y2 as f64 - y1 as f64);
        let x = x1 as f64 + t * (x2 as f64 - x1 as f64);
        println!("  t = {}, x = {}", t, x);
    } else {
        println!("  => NOT FOUND");
    }
}
