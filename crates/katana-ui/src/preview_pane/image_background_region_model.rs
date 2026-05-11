const LARGE_REGION_AREA_DIVISOR: usize = 20;
const MIN_LARGE_REGION_PIXELS: usize = 8;

pub(super) struct LightRegion {
    pixels: Vec<usize>,
    min_x: usize,
    max_x: usize,
    min_y: usize,
    max_y: usize,
}

impl LightRegion {
    pub(super) fn new(start: usize, width: usize) -> Self {
        let x = start % width;
        let y = start / width;
        Self {
            pixels: Vec::new(),
            min_x: x,
            max_x: x,
            min_y: y,
            max_y: y,
        }
    }

    pub(super) fn include(&mut self, index: usize, width: usize) {
        let x = index % width;
        let y = index / width;
        self.pixels.push(index);
        self.min_x = self.min_x.min(x);
        self.max_x = self.max_x.max(x);
        self.min_y = self.min_y.min(y);
        self.max_y = self.max_y.max(y);
    }

    pub(super) fn pixel_indices(self) -> Vec<usize> {
        self.pixels
    }
}

pub(super) fn should_replace_light_region(
    region: &LightRegion,
    width: usize,
    height: usize,
    total: usize,
) -> bool {
    let area_threshold = (total / LARGE_REGION_AREA_DIVISOR).max(MIN_LARGE_REGION_PIXELS);
    let region_width = region.max_x - region.min_x + 1;
    let region_height = region.max_y - region.min_y + 1;
    region.pixels.len() >= area_threshold
        || (region_width > width / 2 && region_height > height / 2)
}
