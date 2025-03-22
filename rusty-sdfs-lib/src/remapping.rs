use crate::vector::VecFloat;

pub fn smoothstep(edge0: VecFloat, edge1: VecFloat, x: VecFloat) -> VecFloat {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smoothstep() {
        assert_eq!(smoothstep(1.0, 3.0, 0.0), 0.0);
        assert_eq!(smoothstep(1.0, 3.0, 1.0), 0.0);
        assert_eq!(smoothstep(1.0, 3.0, 2.0), 0.5);
        assert_eq!(smoothstep(1.0, 3.0, 3.0), 1.0);
        assert_eq!(smoothstep(1.0, 3.0, 4.0), 1.0);
    }
}
