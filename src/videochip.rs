use crate::*;
use core::array::from_fn;

/// A convenient packet of data used to draw a tile as a sprite.
#[derive(Debug, Clone, Copy)]
pub struct DrawBundle {
    pub x: i16,
    pub y: i16,
    pub id: TileID,
    pub flags: TileFlags,
}

/// Main drawing context that manages the screen, tiles, and palette.
#[derive(Debug)]
pub struct VideoChip {
    /// Fixed BG Tilemap
    pub bg: BGMap,
    /// The color rendered if resulting pixel is transparent
    pub bg_color: ColorID,
    /// The main FG palette with 16 colors. Used by sprites.
    pub fg_palette: [Color12Bit; COLORS_PER_PALETTE as usize],
    /// The main BG palette with 16 colors. Used by BG tiles.
    pub bg_palette: [Color12Bit; COLORS_PER_PALETTE as usize],
    /// Local Palettes, 16 with 4 ColorIDs each. Each ID referes to a color in the global palette.
    pub local_palettes: [[ColorID; COLORS_PER_TILE as usize]; LOCAL_PALETTE_COUNT as usize],
    /// Maps i16 coordinates into the u8 range, bringing sprites "outside the screen" into view.
    pub wrap_sprites: bool,
    /// Repeats the BG Map outside its borders
    pub wrap_bg: bool,
    /// Offsets the BG Map and Sprite tiles horizontally
    pub scroll_x: i16,
    /// Offsets the BG Map and Sprite tiles vertically
    pub scroll_y: i16,
    /// Determines which X coordinate triggers the horizontal IRQ callback
    pub horizontal_irq_position: u16,
    /// A callback that can modify the iterator, called once per line.
    /// It is automatically passed to the PixelIterator.
    pub horizontal_irq_callback: Option<HorizontalIRQ>,

    // Pixel data for all tiles, stored as palette indices.
    // pub tiles: &'a [Tile<2>; TILE_COUNT],
    // ---------------------- Main Data ----------------------
    pub(crate) sprite_gen: SpriteGenerator,
    // pub(crate) scanlines: [[Cluster<4>; 256 / PIXELS_PER_CLUSTER as usize]; MAX_LINES],
    pub(crate) w: u16,
    pub(crate) h: u16,
    // ---------------------- Bookkeeping ----------------------
    // view rect cache
    pub(crate) view_left: u16,
    pub(crate) view_top: u16,
    pub(crate) view_right: u16,
    pub(crate) view_bottom: u16,
    // Internal timer. Useful for IRQ manipulation
    frame_count: usize,
    // Next available sprite ID.
    tile_id_head: usize,
    // Next available pixel position in the sprite buffer.
    tile_pixel_head: usize,
    // Next available palette.
    palette_head: u8,
}

impl VideoChip {
    /// Creates a new drawing context with default settings.
    pub fn new(w: u16, h: u16) -> Self {
        assert!(
            h > 7 && h <= MAX_LINES as u16,
            err!("Screen height range is 8 to MAX_LINES")
        );

        let mut result = Self {
            bg: BGMap::new(BG_MAX_COLUMNS, BG_MAX_ROWS),
            // tiles,
            bg_color: GRAY,
            wrap_sprites: true,
            wrap_bg: true,
            fg_palette: [Color12Bit::default(); COLORS_PER_PALETTE as usize],
            bg_palette: [Color12Bit::default(); COLORS_PER_PALETTE as usize],
            local_palettes: [[ColorID(0); COLORS_PER_TILE as usize]; LOCAL_PALETTE_COUNT as usize],
            sprite_gen: SpriteGenerator::new(),
            tile_id_head: 0,
            tile_pixel_head: 0,
            palette_head: 0,
            view_left: 0,
            view_top: 0,
            view_right: w - 1,
            view_bottom: h - 1,
            w,
            h,
            scroll_x: 0,
            scroll_y: 0,
            frame_count: 0,
            // irq: None,
            horizontal_irq_position: 0,
            horizontal_irq_callback: None,
        };
        result.reset_all();

        println!(
            "Total Size of VideoChip:\t{:.1} Kb",
            size_of::<VideoChip>() as f32 / 1024.0
        );
        println!(
            "   Sprite buffers (scanlines):\t{:.1} Kb",
            size_of::<SpriteGenerator>() as f32 / 1024.0
        );
        // println!(
        //     "   Tile Memory:\t\t\t{:.1} Kb",
        //     (result.tiles.len() * size_of::<Tile<2>>()) as f32 / 1024.0
        // );
        println!(
            "   BG Map:\t\t\t{:.1} Kb",
            size_of::<BGMap>() as f32 / 1024.0
        );

        result
    }

    pub fn max_x(&self) -> u16 {
        self.w - 1
    }

    pub fn max_y(&self) -> u16 {
        self.h - 1
    }

    pub fn width(&self) -> u16 {
        self.w
    }

    pub fn height(&self) -> u16 {
        self.h
    }

    pub fn frame_count(&self) -> usize {
        self.frame_count
    }

    /// Does not affect BG or Sprites calculation, but "masks" PixelIter pixels outside
    /// this rectangular area with the BG Color
    pub fn set_viewport(&mut self, left: u16, top: u16, w: u16, h: u16) {
        self.view_left = left;
        self.view_top = top;
        self.view_right = left.saturating_add(w);
        self.view_bottom = top.saturating_add(h);
    }

    /// Resets the chip to its initial state.
    pub fn reset_all(&mut self) {
        self.bg_color = GRAY;
        self.wrap_sprites = true;
        self.frame_count = 0;
        self.reset_scroll();
        self.reset_tiles();
        self.reset_palettes();
        self.reset_bgmap();
        self.reset_viewport();
        self.reset_sprites();
    }

    pub fn reset_tiles(&mut self) {
        self.tile_id_head = 0;
        self.tile_pixel_head = 0;
    }

    pub fn reset_palettes(&mut self) {
        self.fg_palette = from_fn(|i| {
            if i < PALETTE_DEFAULT.len() {
                PALETTE_DEFAULT[i]
            } else {
                Color12Bit::default()
            }
        });
        self.bg_palette = from_fn(|i| {
            if i < PALETTE_DEFAULT.len() {
                PALETTE_DEFAULT[i]
            } else {
                Color12Bit::default()
            }
        });
        self.local_palettes = from_fn(|_| from_fn(|i| ColorID(i as u8)));
        self.palette_head = 0;
    }

    pub fn reset_scroll(&mut self) {
        self.scroll_x = 0;
        self.scroll_y = 0;
    }

    pub fn reset_bgmap(&mut self) {
        self.bg = BGMap::new(BG_MAX_COLUMNS, BG_MAX_ROWS);
    }

    pub fn reset_viewport(&mut self) {
        self.view_left = 0;
        self.view_top = 0;
        self.view_right = self.max_x();
        self.view_bottom = self.max_y();
    }

    pub fn reset_sprites(&mut self) {
        self.sprite_gen.reset();
    }

    pub fn set_palette(&mut self, index: PaletteID, colors: [ColorID; COLORS_PER_TILE as usize]) {
        debug_assert!(
            index.0 < LOCAL_PALETTE_COUNT,
            err!("Invalid local palette index, must be less than PALETTE_COUNT")
        );
        self.local_palettes[index.0 as usize] = colors;
    }

    pub fn push_subpalette(&mut self, colors: [ColorID; COLORS_PER_TILE as usize]) -> PaletteID {
        assert!(self.palette_head < 16, err!("PALETTE_COUNT exceeded"));
        let result = self.palette_head;
        self.local_palettes[self.palette_head as usize] = colors;
        self.palette_head += 1;
        PaletteID(result)
    }

    /// Draws a tile anywhere on the screen using i16 coordinates for convenience. You can
    /// also provide various tile flags, like flipping, and specify a palette id.
    pub fn draw_sprite(&mut self, data: DrawBundle) {
        let size = TILE_SIZE as i16;

        // Handle wrapping
        let wrapped_x: i16;
        let wrapped_y: i16;
        if self.wrap_sprites {
            let screen_x = data.x - self.scroll_x;
            let screen_y = data.y - self.scroll_y;

            let w = self.w as i16;
            let h = self.h as i16;
            let size = TILE_SIZE as i16;

            let adjusted_x = screen_x + size;
            let adjusted_y = screen_y + size;

            // Apply proper modulo wrapping
            let wrapped_adjusted_x =
                ((adjusted_x % (w + size * 2)) + (w + size * 2)) % (w + size * 2);
            let wrapped_adjusted_y =
                ((adjusted_y % (h + size * 2)) + (h + size * 2)) % (h + size * 2);

            // Adjust back to get the final coordinates
            wrapped_x = wrapped_adjusted_x - size;
            wrapped_y = wrapped_adjusted_y - size;
        } else {
            let max_x = self.scroll_x + self.max_x() as i16;
            if data.x + size < self.scroll_x || data.x > max_x {
                return;
            } else {
                wrapped_x = data.x - self.scroll_x;
            }
            let max_y = self.scroll_y + self.max_y() as i16;
            if data.y + size < self.scroll_y || data.y > max_y {
                return;
            } else {
                wrapped_y = data.y - self.scroll_y;
            }
        }

        self.sprite_gen
            .insert(wrapped_x, wrapped_y, self.w, self.h, data.flags, data.id);
    }

    /// Increments or decrements an index in a local palette so that its value
    /// cycles between "min" and "max", which represent colors in the Main FG and BG palettes.
    pub fn color_cycle(&mut self, palette: PaletteID, color: u8, min: u8, max: u8) {
        let color_cycle = &mut self.local_palettes[palette.id()][color as usize].0;
        if max > min {
            *color_cycle += 1;
            if *color_cycle > max {
                *color_cycle = min
            }
        } else {
            *color_cycle -= 1;
            if *color_cycle < min {
                *color_cycle = max
            }
        }
    }

    pub fn start_frame(&mut self) {
        self.frame_count += 1;
        self.reset_sprites();
    }

    /// Returns an iterator over the visible screen pixels, yielding RGB colors for each pixel.
    /// Requires a reference to the Tile array.
    pub fn iter_pixels<'a>(&'a self, tiles: &'a [Tile<2>]) -> PixelIter<'a> {
        PixelIter::new(self, tiles)
    }
}
