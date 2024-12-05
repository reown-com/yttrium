// Thanks ChatGPT: https://chatgpt.com/share/674f376c-3df8-8002-b5b8-6b3da5faa597
pub fn floats_close(a: f64, b: f64, margin: f64) -> bool {
    // Handle the case where both numbers are exactly equal
    if a == b {
        return true;
    }

    let diff = (a - b).abs();
    let max_ab = a.abs().max(b.abs());

    // If the maximum absolute value is zero, both numbers are zero
    if max_ab == 0.0 {
        return true; // Consider zero equal to zero within any margin
    }

    // Calculate the relative difference
    let relative_error = diff / max_ab;

    // Check if the relative error is within the specified margin
    relative_error <= margin
}
