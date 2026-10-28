use core::num::NonZeroU32;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum NativeSurfaceSize {
    Unrenderable,
    Current,
    Changed {
        width: NonZeroU32,
        height: NonZeroU32,
    },
}

impl NativeSurfaceSize {
    pub(crate) fn classify(configured: (u32, u32), actual: (u32, u32)) -> Self {
        let (Some(width), Some(height)) = (NonZeroU32::new(actual.0), NonZeroU32::new(actual.1))
        else {
            return Self::Unrenderable;
        };
        if configured == actual {
            return Self::Current;
        }
        Self::Changed { width, height }
    }
}

#[cfg(test)]
mod tests {
    use super::NativeSurfaceSize;

    #[test]
    fn unchanged_native_size_does_not_reconfigure_the_surface() {
        assert_eq!(
            NativeSurfaceSize::classify((1024, 697), (1024, 697)),
            NativeSurfaceSize::Current
        );
    }

    #[test]
    fn changed_native_size_preserves_both_physical_dimensions() {
        let size = NativeSurfaceSize::classify((1024, 697), (1280, 720));
        let NativeSurfaceSize::Changed { width, height } = size else {
            panic!("changed native dimensions must resize the surface");
        };
        assert_eq!((width.get(), height.get()), (1280, 720));
    }

    #[test]
    fn zero_native_dimension_does_not_configure_an_invalid_surface() {
        for actual in [(0, 697), (1024, 0), (0, 0)] {
            assert_eq!(
                NativeSurfaceSize::classify((1024, 697), actual),
                NativeSurfaceSize::Unrenderable
            );
        }
    }
}
