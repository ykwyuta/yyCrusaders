//! 遠征（シナリオ）ごとの仕組み：始まりの配置（都市の守備兵・野戦軍・英雄・民家）、
//! 第1回十字軍の消耗と回復、民家の襲撃、勝敗の判定

use super::*;

/// 始まりの野戦軍（都市の外にいる兵）
pub struct ArmyDef {
    pub scen: Scen,
    pub nation: usize,
    pub lon: f32,
    pub lat: f32,
    pub units: &'static [(u8, u32)],
}

macro_rules! ar {
    ($s:ident, $n:expr, $lon:expr, $lat:expr, [$($u:expr),* $(,)?]) => {
        ArmyDef { scen: Scen::$s, nation: $n, lon: $lon, lat: $lat, units: &[$($u),*] }
    };
}

pub const ARMIES: &[ArmyDef] = &[
    // ---- 第1回十字軍：諸侯の十字軍は西欧の各地から、民衆十字軍はボスポラスを渡ったところから
    ar!(First, CRUSADERS, 6.3, 50.4, [(UT_KNIGHT, 10), (UT_SERGEANT, 9), (UT_XBOW, 4), (UT_WORKER, 2)]),
    ar!(First, CRUSADERS, 2.3, 48.7, [(UT_KNIGHT, 8), (UT_SERGEANT, 6), (UT_XBOW, 3)]),
    ar!(First, CRUSADERS, 1.7, 43.8, [(UT_KNIGHT, 9), (UT_SERGEANT, 8), (UT_XBOW, 4), (UT_WORKER, 2)]),
    ar!(First, CRUSADERS, 16.7, 41.0, [(UT_KNIGHT, 9), (UT_SERGEANT, 6), (UT_XBOW, 3), (UT_GALLEY, 2)]),
    ar!(First, CRUSADERS, 29.45, 40.62, [(UT_PEOPLE, 18)]),
    ar!(First, FATIMID, 34.3, 31.3, [(UT_SUDANESE, 4), (UT_ARMENIAN_ARCHER, 3), (UT_BEDOUIN, 3)]),
    ar!(First, FATIMID, 31.3, 30.3, [(UT_GHULAM, 3), (UT_SUDANESE, 3), (UT_ARMENIAN_ARCHER, 2)]),
    ar!(First, RUM, 29.8, 40.4, [(UT_HORSE_ARCHER, 6), (UT_GHAZI, 3)]),
    ar!(First, GREAT_SELJUK, 43.2, 36.3, [(UT_GHULAM, 4), (UT_HORSE_ARCHER, 5), (UT_GHAZI, 3)]),
    ar!(First, BYZANTIUM, 29.0, 41.1, [(UT_VARANGIAN, 3), (UT_CATAPHRACT, 2), (UT_TURCOPOLE, 2), (UT_DROMON, 2)]),
    ar!(First, ALEPPO, 36.2, 36.1, [(UT_GHAZI, 4), (UT_ARC, 2)]),
    // ---- 第2回十字軍：ルイ7世はアンティオキアに、コンラート3世はアッコンに上陸した
    ar!(Second, CRUSADERS, 36.0, 36.0, [(UT_KNIGHT, 8), (UT_SERGEANT, 6), (UT_XBOW, 4)]),
    ar!(Second, CRUSADERS, 35.2, 32.8, [(UT_KNIGHT, 6), (UT_SERGEANT, 5), (UT_XBOW, 3)]),
    ar!(Second, CRUSADERS, 35.3, 31.8, [(UT_KNIGHT, 3), (UT_TEMPLAR, 3), (UT_TURCOPOLE, 2)]),
    ar!(Second, ZENGID, 37.25, 36.25, [(UT_MAMLUK, 3), (UT_HORSE_ARCHER, 4), (UT_GHAZI, 3)]),
    ar!(Second, DAMASCUS, 36.35, 33.55, [(UT_GHULAM, 5), (UT_GHAZI, 8), (UT_ARC, 6), (UT_HORSE_ARCHER, 3)]),
    ar!(Second, ZENGID, 36.75, 34.6, [(UT_MAMLUK, 3), (UT_HORSE_ARCHER, 5), (UT_GHAZI, 4)]),
    ar!(Second, BYZANTIUM, 29.0, 41.1, [(UT_VARANGIAN, 2), (UT_CATAPHRACT, 2), (UT_DROMON, 2)]),
    // ---- 第3回十字軍：アッコンの包囲の陣、ティルス、サラディンの陣（テル・カルーバ）
    ar!(Third, CRUSADERS, 35.17, 32.77, [(UT_KNIGHT, 14), (UT_SERGEANT, 14), (UT_XBOW, 10), (UT_TEMPLAR, 4), (UT_HOSPITALLER, 4), (UT_TURCOPOLE, 4), (UT_WORKER, 6)]),
    ar!(Third, CRUSADERS, 35.00, 32.82, [(UT_GALLEY, 3)]),
    ar!(Third, CRUSADERS, 35.22, 33.25, [(UT_KNIGHT, 4), (UT_SERGEANT, 5), (UT_XBOW, 3)]),
    ar!(Third, AYYUBID, 35.47, 32.86, [(UT_HALQA, 4), (UT_KURD, 8), (UT_MAMLUK, 6), (UT_GHAZI, 8), (UT_ARC, 6)]),
    ar!(Third, AYYUBID, 35.07, 32.93, [(UT_GHAZI, 8), (UT_ARC, 5), (UT_NAPHTHA, 3)]),
    ar!(Third, AYYUBID, 36.3, 33.5, [(UT_MAMLUK, 3), (UT_KURD, 3)]),
    ar!(Third, AYYUBID, 35.25, 31.82, [(UT_GHAZI, 6), (UT_ARC, 4), (UT_KURD, 3)]),
    ar!(Third, AYYUBID, 35.45, 33.92, [(UT_SHINI, 2)]),
];

/// 民家が焼かれてから建て直されるまで（秒）
pub const VILLAGE_REGROW: f32 = 240.0;
/// 民家を焼いたとき、近くの自軍が癒える割合と範囲
pub const RAID_HEAL: f32 = 0.30;
pub const RAID_RANGE: f32 = 6.0;

/// 勝敗の結果（遠征ごとの記録）
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct ScenState {
    /// 第1回：十字軍がエルサレムを占領した時（経過年）
    pub jeru_since: Option<f32>,
    /// 判定の拍子
    pub check_t: f32,
    /// 部族の動き・傭兵の満了を見る拍子（第2回の1年＝24秒ごと）
    pub trade_t: f32,
    /// 第1回：十字軍の兵を最後に討った国
    pub last_killer: u8,
}

impl World {
    pub fn scen_def(&self) -> &'static ScenDef {
        self.scen.def()
    }

    /// 遠征の始まりの年
    pub fn start_ad(&self) -> f32 {
        self.scen.def().start_ad
    }

    /// 遠征の終わりの年
    pub fn end_ad(&self) -> f32 {
        self.scen.def().end_ad
    }

    /// 都市を持たずに兵だけで戦う国（第1回十字軍の十字軍）
    pub fn army_nation(&self, n: usize) -> bool {
        self.scen == Scen::First && n == CRUSADERS
    }

    /// 兵も民も補充できない国（第1回十字軍の十字軍）
    pub fn no_reinforce(&self, n: usize) -> bool {
        self.army_nation(n)
    }

    /// 第1回：セルジューク系の国（ルーム・大セルジューク・ダニシュメンド・ダマスカス・アレッポ・アルトゥク）は、
    /// 自領を通るファーティマ朝・十字軍の兵を、宣戦していなくても迎え撃つ（迎え撃たれた側も、その地で戦い返せる）
    pub fn skirmish(&self, a: usize, b: usize, p: V2) -> bool {
        if self.scen != Scen::First || a >= NATIONS.len() || b >= NATIONS.len() {
            return false;
        }
        let seljuk = |n: usize| matches!(n, RUM | GREAT_SELJUK | DANISHMEND | DAMASCUS | ALEPPO | ARTUQID);
        let outsider = |n: usize| n == FATIMID || n == CRUSADERS;
        let host = if seljuk(a) && outsider(b) {
            a
        } else if seljuk(b) && outsider(a) {
            b
        } else {
            return false;
        };
        let (x, y) = p.tile();
        Map::inb(x, y) && self.owner[Map::idx(x, y)] as usize == host
    }

    /// 城邑の領域の半径（地図の縮尺に合わせる）
    pub fn city_radius(&self, k: BKind) -> i32 {
        let m = match self.scen {
            Scen::First => 1.3,
            Scen::Second => 1.5,
            Scen::Third => 1.7,
        };
        (k.radius() as f32 * m).round() as i32
    }

    /// 兵の上限の最大（遠征の規模による）
    pub fn cap_max(&self) -> u32 {
        match self.scen {
            Scen::First => 40,
            Scen::Second => 45,
            Scen::Third => 100,
        }
    }

    /// いまの日付（年と月）
    pub fn date_str(&self) -> String {
        let ad = self.start_ad() + self.years;
        let y = ad.floor() as i32;
        let m = ((ad - y as f32) * 12.0).floor() as i32 + 1;
        tf!("{}年{}月", "{1} {0}", "{}年{}月", y, if crate::lang::lang() == crate::lang::Lang::En { MONTHS[(m - 1).clamp(0, 11) as usize].to_string() } else { m.clamp(1, 12).to_string() })
    }

    /// 都市の名前（実在の都市なら）
    pub fn city_name(&self, b: &Building) -> Option<&'static str> {
        cities().get(b.site as usize).map(|c| c.name)
    }

    /// 名前の都市の建物
    pub fn city_bld(&self, name: &str) -> Option<usize> {
        let k = city_index(name)? as u16;
        (0..self.blds.len()).find(|&i| self.blds[i].alive && self.blds[i].kind.city() && self.blds[i].site == k)
    }

    // ------------------------------------------------------------ 始まりの配置
    pub(super) fn setup_scenario(&mut self) {
        let s = self.scen;
        let d = s.def();
        let res = match s {
            Scen::First => [300.0, 300.0, 200.0],
            Scen::Second => [300.0, 300.0, 150.0],
            Scen::Third => [800.0, 600.0, 500.0],
        };
        for n in 0..NATIONS.len() {
            self.nations[n].res = res;
        }
        // 都市
        for (k, c) in d.cities.iter().enumerate() {
            if !self.nations[c.owner].alive {
                continue;
            }
            let kind = if c.cap { BKind::Capital } else { BKind::Town };
            if let Some(h) = self.found_city(c.owner, c.lon, c.lat, kind) {
                self.blds[h.idx as usize].site = k as u16;
                if c.cap {
                    self.nations[c.owner].capital = Some(h);
                    self.nations[c.owner].home = self.blds[h.idx as usize].center();
                }
            }
        }
        // 都のない国は最初の都市を都にする（第1回の十字軍は都を持たない）
        for n in 0..NATIONS.len() {
            if !self.nations[n].alive || self.nations[n].capital.is_some() {
                continue;
            }
            if let Some(i) = (0..self.blds.len()).find(|&i| self.blds[i].alive && self.blds[i].nation as usize == n && self.blds[i].kind.city()) {
                self.blds[i].kind = BKind::Capital;
                let h = self.bh(i);
                self.nations[n].capital = Some(h);
                self.nations[n].home = self.blds[i].center();
            }
        }
        self.recompute_territory();
        // 兵営（都と、遠征ごとに決めた都市）
        let sites: Vec<usize> = (0..self.blds.len())
            .filter(|&i| {
                let b = &self.blds[i];
                b.alive && b.kind.city() && (b.kind == BKind::Capital || cities().get(b.site as usize).map_or(false, |c| d.barracks.contains(&c.name)))
            })
            .collect();
        for i in sites {
            let (c, n) = (self.blds[i].center(), self.blds[i].nation as usize);
            'spot: for r in 3..7i32 {
                for dy in -r..=r {
                    for dx in -r..=r {
                        if dx.abs() != r && dy.abs() != r {
                            continue;
                        }
                        let (tx, ty) = (c.x as i32 + dx - 1, c.y as i32 + dy - 1);
                        if self.can_place(n, BKind::Barracks, tx, ty).is_ok() && self.inland_spot(tx, ty) {
                            self.spawn_building(n, BKind::Barracks, tx, ty, true);
                            break 'spot;
                        }
                    }
                }
            }
        }
        // 城壁の要塞
        for &(name, cnt) in d.forts {
            let Some(i) = self.city_bld(name) else { continue };
            let (c, n) = (self.blds[i].center(), self.blds[i].nation as usize);
            let mut placed = 0;
            'fort: for r in 3..8i32 {
                for dy in -r..=r {
                    for dx in -r..=r {
                        if dx.abs() != r && dy.abs() != r || (dx + dy + r) % 3 != 0 {
                            continue;
                        }
                        let (tx, ty) = (c.x as i32 + dx - 1, c.y as i32 + dy - 1);
                        let p = v2(tx as f32 + 1.0, ty as f32 + 1.0);
                        if self.footprint_free(tx, ty) && self.inland_spot(tx, ty) && self.owner[Map::idx(tx, ty)] as usize == n && !self.blds.iter().any(|b| b.alive && b.kind == BKind::Fort && b.center().dist(p) < 4.0) {
                            self.spawn_building(n, BKind::Fort, tx, ty, true);
                            placed += 1;
                            if placed >= cnt {
                                break 'fort;
                            }
                        }
                    }
                }
            }
        }
        // 守備兵と民
        let (cw, cs, tw, ts) = match s {
            Scen::First => (3, 3, 1, 1),
            Scen::Second => (4, 3, 1, 2),
            Scen::Third => (4, 3, 1, 1),
        };
        for i in 0..self.blds.len() {
            let b = &self.blds[i];
            if !b.alive || !b.kind.city() {
                continue;
            }
            let (n, cap) = (b.nation as usize, b.kind == BKind::Capital);
            let h = self.bh(i);
            let r = ROSTERS[n];
            let land: Vec<u8> = r.iter().copied().filter(|&u| udef(u).class != UnitKind::Ship).collect();
            let (nw, ns) = if cap { (cw, cs) } else { (tw, ts) };
            let mut list = vec![UT_WORKER; nw];
            for k in 0..ns {
                list.push(land[k % land.len().min(3).max(1)]);
            }
            for ut in list {
                if let Some(sp) = self.spawn_point(h) {
                    let uh = self.spawn_unit(n, ut, sp, 1.0);
                    if ut == UT_WORKER {
                        self.auto_gather(uh);
                    }
                }
            }
        }
        // 野戦軍
        for a in ARMIES.iter().filter(|a| a.scen == s) {
            if !self.nations[a.nation].alive {
                continue;
            }
            let (x, y) = ll_to_tile(a.lon, a.lat);
            let c = v2(x, y);
            let mut k = 0;
            for &(ut, cnt) in a.units {
                for _ in 0..cnt {
                    let ship = udef(ut).class == UnitKind::Ship;
                    let p = if ship {
                        self.shore_water_near(c, 14).map(|w| w + v2((k % 3) as f32 * 0.7 - 0.7, 0.0))
                    } else {
                        let off = v2((k % 6) as f32 * 0.8 - 2.0, (k / 6) as f32 * 0.8 - 1.5);
                        Some(self.passable_at_or_near(c + off))
                    };
                    if let Some(p) = p {
                        if !ship || self.passable_l(p, true) {
                            // 第1回の十字軍は出発の時が最も強い（補充はできない）
                            let pw = if self.army_nation(a.nation) && ut != UT_PEOPLE && ut != UT_WORKER { 1.25 } else { 1.0 };
                            self.spawn_unit(a.nation, ut, p, pw);
                        }
                    }
                    k += 1;
                }
            }
        }
        self.spawn_heroes();
        // 都のない国（第1回の十字軍）は、最初の野戦軍の居場所を本拠とする
        for n in 0..NATIONS.len() {
            if self.nations[n].alive && self.nations[n].capital.is_none() {
                if let Some(a) = ARMIES.iter().find(|a| a.scen == s && a.nation == n) {
                    let (x, y) = ll_to_tile(a.lon, a.lat);
                    self.nations[n].home = v2(x, y);
                }
            }
        }
        // 民家
        self.place_villages(d.villages);
        // 交戦
        for &(a, b) in d.wars {
            if self.nations[a].alive && self.nations[b].alive {
                self.war[a][b] = true;
                self.war[b][a] = true;
            }
        }
    }

    /// 建物（左上 tx, ty）のまわり1マスに海が無いか（細い陸橋や海岸の道をふさがないように）
    fn inland_spot(&self, tx: i32, ty: i32) -> bool {
        for y in ty - 1..=ty + 2 {
            for x in tx - 1..=tx + 2 {
                if !Map::inb(x, y) || self.map.terrain[Map::idx(x, y)].water() {
                    return false;
                }
            }
        }
        true
    }

    /// 都市のまわりに民家を置く
    fn place_villages(&mut self, per_city: u32) {
        let mut rng = Rng::new(self.rng.0 ^ 0x5151);
        let centers: Vec<(V2, usize)> = self.blds.iter().filter(|b| b.alive && b.kind.city()).map(|b| (b.center(), b.nation as usize)).collect();
        for (c, n) in centers {
            let mut placed = 0;
            for _ in 0..40 {
                if placed >= per_city {
                    break;
                }
                let ang = rng.range(0.0, std::f32::consts::TAU);
                let d = rng.range(4.0, 7.5);
                let (tx, ty) = ((c.x + ang.cos() * d) as i32 - 1, (c.y + ang.sin() * d) as i32 - 1);
                if !self.footprint_free(tx, ty) || !self.inland_spot(tx, ty) {
                    continue;
                }
                let p = v2(tx as f32 + 1.0, ty as f32 + 1.0);
                if self.blds.iter().any(|b| b.alive && b.center().dist(p) < 3.5) || !self.same_land(p, c) {
                    continue;
                }
                let o = self.owner[Map::idx(tx, ty)];
                let owner = if o != NONE { o as usize } else { n };
                self.spawn_building(owner, BKind::Village, tx, ty, true);
                placed += 1;
            }
        }
    }

    /// 民家の持ち主を領土の持ち主に合わせる（領土を組み直したとき）
    pub(super) fn sync_villages(&mut self) {
        for b in self.blds.iter_mut() {
            if b.alive && b.kind == BKind::Village {
                let c = b.center();
                let (x, y) = (c.x as i32, c.y as i32);
                if Map::inb(x, y) {
                    let o = self.owner[Map::idx(x, y)];
                    if o != NONE && (o as usize) < NATIONS.len() {
                        b.nation = o;
                    }
                }
            }
        }
    }

    /// 民家を焼いた：略奪品を得て、近くの兵が癒える（十字軍はこれで飢えをしのいだ）
    pub(super) fn raid_village(&mut self, bi: usize, an: usize) {
        let (c, tx, ty, vn) = {
            let b = &self.blds[bi];
            (b.center(), b.tx, b.ty, b.nation as usize)
        };
        self.remove_building(bi);
        self.ruins.push((tx, ty, self.time + VILLAGE_REGROW));
        if an >= NATIONS.len() {
            return;
        }
        let loot = [60.0, 20.0, 15.0];
        for k in 0..3 {
            self.nations[an].res[k] += loot[k];
        }
        let mut healed = 0;
        for u in self.units.iter_mut() {
            if u.on_map() && u.nation as usize == an && !u.kind.naval() && u.pos.dist(c) <= RAID_RANGE {
                let m = u.max_hp();
                if u.hp < m {
                    u.hp = (u.hp + m * RAID_HEAL).min(m);
                    healed += 1;
                }
            }
        }
        if an == self.player {
            self.push_msg(tf!("民家を襲い、食糧を奪った（兵{}人が力を取り戻した）", "Raided a village and seized its food ({} troops recovered)", "袭击民宅，夺得粮食（{}名士兵恢复了体力）", healed), [0.95, 0.8, 0.5]);
        } else if vn == self.player {
            self.push_msg(t!("民家が襲われ、焼かれた", "One of our villages was raided and burned", "民宅遭到袭击并被焚毁").into(), [1.0, 0.5, 0.4]);
        }
    }

    /// 焼かれた民家を建て直す
    fn regrow_villages(&mut self) {
        let now = self.time;
        let mut keep = Vec::new();
        let ruins = std::mem::take(&mut self.ruins);
        for (tx, ty, t) in ruins {
            if t > now {
                keep.push((tx, ty, t));
                continue;
            }
            let o = self.owner[Map::idx(tx, ty)];
            if o == NONE || o as usize >= NATIONS.len() || !self.footprint_free(tx, ty) {
                // 持ち主のいない土地は荒れたまま。しばらくしてまた見る
                keep.push((tx, ty, now + VILLAGE_REGROW * 0.5));
                continue;
            }
            self.spawn_building(o as usize, BKind::Village, tx, ty, true);
        }
        self.ruins = keep;
    }

    // ------------------------------------------------------------ 第1回十字軍の消耗と回復
    /// 十字軍は進軍で疲れ、占領地で癒える。ビザンツ領では騎士だけが癒える（民衆十字軍は癒えない）
    pub(super) fn crusade_attrition(&mut self, dt: f32) {
        if self.scen != Scen::First || !self.nations[CRUSADERS].alive {
            return;
        }
        let byz_friend = !self.war[CRUSADERS][BYZANTIUM];
        for i in 0..self.units.len() {
            let u = &self.units[i];
            if !u.on_map() || u.nation as usize != CRUSADERS || u.kind.naval() {
                continue;
            }
            let (x, y) = u.pos.tile();
            let o = if Map::inb(x, y) { self.owner[Map::idx(x, y)] as usize } else { NONE as usize };
            let people = u.ut == UT_PEOPLE;
            let moving = !matches!(u.order, Order::Idle);
            let m = u.max_hp();
            let rate = if o == CRUSADERS {
                0.010
            } else if o == BYZANTIUM && byz_friend && !people {
                0.006
            } else if moving {
                -0.0012
            } else {
                -0.0005
            };
            let u = &mut self.units[i];
            u.hp = (u.hp + m * rate * dt).min(m);
            if u.hp <= 0.0 {
                let hero = u.hero;
                self.kill_unit(i);
                if (hero as usize) < HEROES.len() {
                    let t = tf!("{} は飢えと病に倒れた", "{} succumbed to hunger and disease", "{}倒在了饥饿与疾病之中", tx(HEROES[hero as usize].name));
                    self.push_msg(t, [1.0, 0.7, 0.5]);
                }
            }
        }
    }

    // ------------------------------------------------------------ 勝敗
    pub(super) fn scenario_tick(&mut self, dt: f32) {
        self.crusade_attrition(dt);
        self.st.check_t -= dt;
        if self.st.check_t > 0.0 {
            return;
        }
        self.st.check_t = 0.5;
        self.regrow_villages();
        // 史実の出来事は月の単位で起こる
        self.run_history();
        self.st.trade_t += 0.5;
        if self.st.trade_t >= 24.0 {
            self.st.trade_t -= 24.0;
            self.yearly_trade();
        }
        // 兵だけの国：兵が尽きれば滅ぶ
        for n in 0..NATIONS.len() {
            if self.nations[n].alive && self.army_nation(n) {
                let has_city = self.blds.iter().any(|b| b.alive && b.nation as usize == n && b.kind.city() && b.done());
                let has_troops = self.units.iter().any(|u| u.alive && u.nation as usize == n && u.kind.military() && !u.kind.naval());
                if !has_city && !has_troops {
                    let k = self.st.last_killer as usize;
                    let by = if k < NATIONS.len() && k != n { k } else { (0..NATIONS.len()).find(|&o| self.war[n][o]).unwrap_or(n) };
                    self.eliminate(n, by);
                }
            }
        }
        if self.game_over.is_none() && !self.ai_player_only() {
            self.check_victory();
        }
    }

    /// 撮影・試験用に全ての国を AI に任せているか
    fn ai_player_only(&self) -> bool {
        false
    }

    fn end_game(&mut self, win: bool, draw: bool, text: String) {
        if self.game_over.is_some() {
            return;
        }
        self.draw = draw;
        self.game_over = Some((win, text));
    }

    fn check_victory(&mut self) {
        let p = self.player;
        let ad = self.start_ad() + self.years;
        match self.scen {
            Scen::First => {
                let jer = self.city_bld("エルサレム");
                let held = jer.map_or(false, |i| self.blds[i].nation as usize == CRUSADERS);
                // 十字軍が勝ったか（Some(true)）、イスラーム勢力が勝ったか（Some(false)）
                let mut result: Option<(bool, String)> = None;
                match (held, self.st.jeru_since) {
                    (true, None) => {
                        self.st.jeru_since = Some(self.years);
                        let t = tf!("十字軍がエルサレムを占領した（{}）。5年守り抜けば十字軍の勝利", "The crusaders have taken Jerusalem ({}). If they hold it for 5 years, they win", "十字军攻占了耶路撒冷（{}）。坚守5年即为十字军胜利", self.date_str());
                        self.push_msg(t.clone(), [1.0, 0.85, 0.4]);
                        self.queue_banner(Banner { title: t!("エルサレムの陥落", "The Fall of Jerusalem", "耶路撒冷陷落").into(), text: t, t: 7.0 });
                    }
                    (true, Some(since)) if self.years - since >= 5.0 => {
                        result = Some((true, t!("十字軍はエルサレムを5年守り抜き、聖地にラテン人の王国を築いた！", "The crusaders held Jerusalem for five years and founded a Latin kingdom in the Holy Land!", "十字军坚守耶路撒冷五年，在圣地建立了拉丁人的王国！").into()));
                    }
                    (false, Some(_)) => {
                        result = Some((false, t!("十字軍はエルサレムを守り切れなかった。聖地はイスラームの手に戻った", "The crusaders could not hold Jerusalem. The Holy City has returned to Muslim hands", "十字军未能守住耶路撒冷，圣城回到了伊斯兰手中").into()));
                    }
                    (false, None) if ad >= HIST_JERUSALEM => {
                        result = Some((false, t!("1099年7月までに十字軍はエルサレムに至らなかった。十字軍は四散した", "The crusaders failed to reach Jerusalem by July 1099. The crusade has dissolved", "十字军未能在1099年7月前抵达耶路撒冷，十字军四散了").into()));
                    }
                    _ => {}
                }
                if !self.nations[CRUSADERS].alive {
                    result = Some((false, t!("十字軍は全滅した", "The crusade has been annihilated", "十字军全军覆没").into()));
                }
                if let Some((crusade_won, why)) = result {
                    if p == CRUSADERS {
                        self.end_game(crusade_won, false, why);
                    } else {
                        let ok = !crusade_won && self.nations[p].alive;
                        self.end_game(ok, false, why);
                    }
                }
            }
            Scen::Second => {
                if p == CRUSADERS {
                    if let Some(i) = self.city_bld("ダマスカス") {
                        if self.blds[i].nation as usize == CRUSADERS {
                            self.end_game(true, false, t!("十字軍はダマスカスを陥れた！ 第2回十字軍は聖地に新たな勝利をもたらした", "The crusaders have taken Damascus! The Second Crusade brings a new triumph to the Holy Land", "十字军攻陷了大马士革！第二次十字军为圣地带来了新的胜利").into());
                            return;
                        }
                    }
                    if ad >= self.end_ad() {
                        self.end_game(false, false, t!("1250年までにダマスカスは落ちなかった", "Damascus did not fall before 1250", "直到1250年大马士革都未陷落").into());
                    }
                } else if ad >= self.end_ad() {
                    let best = (0..NATIONS.len()).filter(|&n| self.nations[n].alive).max_by_key(|&n| self.nations[n].tiles);
                    if let Some(b) = best {
                        let (mine, top) = (self.nations[p].tiles, self.nations[b].tiles);
                        if b == p || mine >= top {
                            self.end_game(true, false, tf!("1250年、{} は最も広い版図（{}）を築いた！", "In 1250, {} holds the largest realm ({} tiles)!", "1250年，{}拥有最广阔的版图（{}）！", tx(self.nations[p].name), mine));
                        } else {
                            self.end_game(false, false, tf!("1250年、最も広い版図を築いたのは {}（{}）だった。わが国は {}", "In 1250, the largest realm belongs to {} ({} tiles). Ours: {}", "1250年，版图最广的是{}（{}）。我国为{}", tx(self.nations[b].name), top, mine));
                        }
                    }
                }
            }
            Scen::Third => {
                let mut cnt = [0usize; 2];
                for b in self.blds.iter() {
                    if b.alive && b.kind.city() && cities().get(b.site as usize).map_or(false, |c| c.coast) {
                        if b.nation as usize == CRUSADERS {
                            cnt[0] += 1;
                        } else if b.nation as usize == AYYUBID {
                            cnt[1] += 1;
                        }
                    }
                }
                let crus_out = cnt[0] == 0 || !self.nations[CRUSADERS].alive;
                let ayy_out = cnt[1] == 0 || !self.nations[AYYUBID].alive;
                if ayy_out && !crus_out {
                    let t = t!("十字軍はレバントの沿岸からアイユーブ朝を駆逐した！", "The crusaders have driven the Ayyubids from the Levantine coast!", "十字军将阿尤布王朝逐出了黎凡特沿岸！");
                    self.end_game(p == CRUSADERS, false, t.into());
                } else if crus_out && !ayy_out {
                    let t = t!("サラディンは十字軍をレバントの沿岸から海へ追い落とした！", "Saladin has driven the crusaders from the Levantine coast into the sea!", "萨拉丁将十字军从黎凡特沿岸赶入大海！");
                    self.end_game(p == AYYUBID, false, t.into());
                } else if ad >= self.end_ad() {
                    let t = tf!("1192年9月、ヤッファの和約が結ばれた。沿岸は十字軍が{}都市、アイユーブ朝が{}都市を保ち、巡礼者はエルサレムを訪れることを許された", "September 1192: the Treaty of Jaffa is signed. The crusaders keep {} coastal cities and the Ayyubids {}; pilgrims may visit Jerusalem", "1192年9月，双方缔结雅法和约。沿岸十字军保有{}座城市，阿尤布王朝保有{}座，朝圣者获准前往耶路撒冷", cnt[0], cnt[1]);
                    self.end_game(false, true, t);
                }
            }
        }
    }
}

impl Default for ScenState {
    fn default() -> Self {
        ScenState { jeru_since: None, check_t: 0.0, trade_t: 0.0, last_killer: NONE }
    }
}

/// 史実のエルサレム占領（1099年7月15日）
pub const HIST_JERUSALEM: f32 = 1099.54;

const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
