//! Generated period artwork. Slots are normalized at build-authoring time, never at runtime.
use super::*;

const SIZE: usize = 192;
const CELL: usize = SIZE * SIZE * 4;
const ATLASES: [&[u8]; 10] = [
    include_bytes!("../../assets/period/steppe.bgra"),
    include_bytes!("../../assets/period/song.bgra"),
    include_bytes!("../../assets/period/japan.bgra"),
    include_bytes!("../../assets/period/goryeo.bgra"),
    include_bytes!("../../assets/period/islamic.bgra"),
    include_bytes!("../../assets/period/latin.bgra"),
    include_bytes!("../../assets/period/orthodox.bgra"),
    include_bytes!("../../assets/period/southern.bgra"),
    include_bytes!("../../assets/period/tibet.bgra"),
    include_bytes!("../../assets/period/ainu.bgra"),
];
const EXTRA: &[u8] = include_bytes!("../../assets/period/extra.bgra");
const HEROES_RAW: &[u8] = include_bytes!("../../assets/period/heroes.bgra");

/// Nation order is the simulation's NATIONS/ROSTERS order. Architecture follows origin.
pub fn culture(n: usize) -> usize {
    crate::geo::NATIONS.get(n).map_or(5, |d| d.culture as usize)
}

#[derive(Default)]
pub(super) struct PeriodArt {
    normal: Vec<Option<ID2D1Bitmap>>,
    flipped: Vec<Option<ID2D1Bitmap>>,
    fog: Vec<Option<ID2D1Bitmap>>,
    heroes: Vec<Option<ID2D1Bitmap>>,
    heroes_flip: Vec<Option<ID2D1Bitmap>>,
}

impl PeriodArt {
    pub(super) fn load(&mut self, gfx: &Gfx) {
        self.normal.clear(); self.flipped.clear(); self.fog.clear();
        self.heroes.clear(); self.heroes_flip.clear();
        for raw in ATLASES.into_iter().flat_map(|a| a.chunks_exact(CELL)).chain(EXTRA.chunks_exact(CELL)) {
            self.normal.push(gfx.create_bitmap(SIZE as u32, SIZE as u32, &load_raw_sprite_bgra(raw, SIZE, false, 1.0)));
            self.flipped.push(gfx.create_bitmap(SIZE as u32, SIZE as u32, &load_raw_sprite_bgra(raw, SIZE, true, 1.0)));
            self.fog.push(gfx.create_bitmap(SIZE as u32, SIZE as u32, &load_raw_sprite_bgra(raw, SIZE, false, 0.52)));
        }
        for raw in HEROES_RAW.chunks_exact(128 * 128 * 4) {
            self.heroes.push(gfx.create_bitmap(128, 128, &load_raw_sprite_bgra(raw, 128, false, 1.0)));
            self.heroes_flip.push(gfx.create_bitmap(128, 128, &load_raw_sprite_bgra(raw, 128, true, 1.0)));
        }
    }
    pub(super) fn get(&self, idx: usize, flip: bool, fog: bool) -> Option<&ID2D1Bitmap> {
        (if fog { &self.fog } else if flip { &self.flipped } else { &self.normal }).get(idx).and_then(Option::as_ref)
    }
}

impl Art {
    pub fn national_unit(&self, nation: usize, ut: u8, flip: bool) -> Option<&ID2D1Bitmap> {
        use crate::world::udef;
        // Unique troops retain their equipment even when recruited by a subject's overlord.
        // 文化ごとの絵：4=イスラーム 5=ラテン 6=正教。兵種に固有の装いがあればそれを使う
        let c = match ut {
            5..=10 => 5, 11..=13 => 6, 4 | 17..=26 => 4, _ => culture(nation),
        };
        if udef(ut).class == UnitKind::Ship {
            return if flip { self.galley_flip.as_ref() } else { self.galley.as_ref() };
        }
        let slot = match ut {
            9 | 10 | 12 | 18 | 25 => 5,
            4 | 11 => 3,
            _ => match udef(ut).class { UnitKind::Worker => 0, UnitKind::Infantry => 1, UnitKind::Archer => 2, _ => 4 },
        };
        self.period.get(c * 12 + slot, flip, false)
    }
    pub fn national_building(&self, origin: usize, kind: BKind, fog: bool) -> Option<&ID2D1Bitmap> {
        let slot = match kind { BKind::Capital => 6, BKind::Town | BKind::Village => 7, BKind::Barracks => 8, BKind::Farm => 9, BKind::Fort => 10, BKind::Port => 11 };
        self.period.get(culture(origin) * 12 + slot, false, fog)
    }
    /// The hero's homeland determines dress, independently of the summoning nation.
    pub fn period_hero(&self, hero: usize, flip: bool) -> Option<&ID2D1Bitmap> {
        let art = crate::world::HEROES.get(hero)?.art as usize;
        (if flip { &self.period.heroes_flip } else { &self.period.heroes }).get(art).and_then(Option::as_ref)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn atlas_layout_and_nation_coverage() {
        for a in ATLASES { assert_eq!(a.len(), CELL * 12); }
        assert_eq!(EXTRA.len(), CELL * 8);
        assert_eq!(HEROES_RAW.len(), crate::world::HERO_ART * 128 * 128 * 4);
        for h in crate::world::HEROES { assert!((h.art as usize) < crate::world::HERO_ART, "{}", h.name); }
        for n in 0..crate::geo::NATIONS.len() { assert!(culture(n) < ATLASES.len()); }
    }
}
