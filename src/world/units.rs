//! 兵種：国（文化）ごとに作れる兵が違う。服属させた国（その国が築いた城邑を領する）の兵も作れる。
//! 船はイスラームの沿岸国家（ファーティマ朝・アイユーブ朝）・イタリアの海の共和国・ビザンツ帝国だけが造れる（占領しても造れない）。

use super::*;

/// 兵種の定義
pub struct UnitDef {
    pub name: &'static str,
    /// 戦い方の大分類（相性・見た目・維持費の区分に使う）
    pub class: UnitKind,
    pub hp: f32,
    pub atk: f32,
    /// 射程（1.5 より長ければ遠くから射る）
    pub range: f32,
    pub cd: f32,
    pub speed: f32,
    /// [糧, 木, 鉄]
    pub cost: [f32; 3],
    pub time: f32,
    /// 建物への攻撃力の倍率
    pub vs_bld: f32,
    /// 特に強い相手（大分類, 倍率）
    pub extra: Option<(UnitKind, f32)>,
    pub desc: &'static str,
}

macro_rules! ud {
    ($n:expr, $c:ident, $hp:expr, $atk:expr, $r:expr, $cd:expr, $sp:expr, $cost:expr, $t:expr, $vb:expr, $ex:expr, $d:expr) => {
        UnitDef { name: $n, class: UnitKind::$c, hp: $hp, atk: $atk, range: $r, cd: $cd, speed: $sp, cost: $cost, time: $t, vs_bld: $vb, extra: $ex, desc: $d }
    };
}

const CAV: Option<(UnitKind, f32)> = Some((UnitKind::Cavalry, 1.8));

pub const UNIT_DEFS: &[UnitDef] = &[
    // 0
    ud!("民", Worker, 40.0, 3.0, 0.75, 1.2, 1.9, [50.0, 0.0, 0.0], 8.0, 0.4, None, "資源を集め、建物を建てる"),
    // 1〜3 共通
    ud!("歩兵", Infantry, 120.0, 12.0, 0.75, 1.2, 1.9, [60.0, 0.0, 25.0], 10.0, 1.0, None, "騎兵に強い歩兵。城攻めに優れる"),
    ud!("弓兵", Archer, 75.0, 9.0, 4.5, 1.6, 1.75, [50.0, 35.0, 15.0], 11.0, 0.35, None, "遠くから射る。歩兵に強いが騎兵に弱い"),
    ud!("騎兵", Cavalry, 140.0, 14.0, 0.75, 1.4, 3.2, [80.0, 45.0, 40.0], 14.0, 0.7, None, "速い突撃兵。弓兵と民に強い"),
    // 4 トルコ
    ud!("トルコ騎射兵", Cavalry, 105.0, 9.0, 3.8, 1.5, 3.5, [70.0, 40.0, 25.0], 12.0, 0.3, None, "馬上から弓を射るトルコ人の遊牧騎兵。速く、遠くから射る"),
    // 5〜11 ラテン
    ud!("騎士", Cavalry, 185.0, 18.0, 0.75, 1.5, 2.9, [110.0, 45.0, 60.0], 16.0, 0.7, None, "重い鎖帷子と長槍で突撃する西欧の騎士"),
    ud!("従士歩兵", Infantry, 130.0, 13.0, 0.75, 1.2, 1.9, [60.0, 0.0, 30.0], 10.0, 1.0, CAV, "槍と盾で戦う徒歩の従士。騎兵を止める"),
    ud!("弩兵", Archer, 80.0, 11.0, 4.6, 2.0, 1.7, [55.0, 40.0, 25.0], 12.0, 0.4, None, "鎧を射抜くクロスボウ兵"),
    ud!("民衆十字軍", Infantry, 85.0, 8.0, 0.75, 1.3, 1.9, [30.0, 0.0, 5.0], 8.0, 0.8, None, "隠者ピエールに従った貧しい民の十字軍。数は多いが弱い"),
    ud!("テンプル騎士", Cavalry, 205.0, 19.0, 0.75, 1.5, 2.9, [120.0, 50.0, 65.0], 17.0, 0.8, None, "聖地の巡礼路を守る騎士修道会の騎士"),
    ud!("ホスピタル騎士", Cavalry, 200.0, 18.0, 0.75, 1.5, 2.9, [115.0, 50.0, 65.0], 17.0, 0.8, None, "聖ヨハネ騎士団の騎士。病院から始まり城を守る"),
    ud!("トルコポーレ", Cavalry, 110.0, 9.0, 3.6, 1.5, 3.4, [70.0, 40.0, 25.0], 12.0, 0.3, None, "十字軍とビザンツに仕える軽装の騎射兵"),
    // 12〜14 ビザンツ
    ud!("ヴァリャーグ親衛隊", Infantry, 160.0, 16.0, 0.75, 1.2, 1.9, [90.0, 0.0, 45.0], 13.0, 1.1, None, "大斧を振るう北方人の皇帝親衛隊"),
    ud!("カタフラクト", Cavalry, 190.0, 15.0, 0.75, 1.5, 2.8, [100.0, 40.0, 60.0], 16.0, 0.7, None, "人馬ともに鎧をまとうローマの重騎兵"),
    ud!("ドロモン", Ship, 360.0, 13.0, 4.2, 1.8, 2.8, [70.0, 170.0, 50.0], 18.0, 0.6, Some((UnitKind::Ship, 1.7)), "ギリシアの火を放つビザンツの軍船。兵6人を乗せて海を渡る"),
    // 15〜16 船
    ud!("ガレー船", Ship, 320.0, 11.0, 4.2, 1.7, 3.1, [60.0, 160.0, 40.0], 17.0, 0.5, None, "櫂で進むイタリアの軍船。速く、兵6人を乗せて海を渡る"),
    ud!("シーニー船", Ship, 330.0, 11.0, 4.2, 1.8, 2.9, [60.0, 160.0, 40.0], 17.0, 0.5, None, "イスラームの国々の軍船。兵6人を乗せて海を渡る"),
    // 17〜26 イスラーム
    ud!("グラーム騎兵", Cavalry, 160.0, 15.0, 0.75, 1.4, 3.1, [90.0, 40.0, 50.0], 15.0, 0.7, None, "スルタンやアミールに仕える奴隷出身の騎兵"),
    ud!("マムルーク", Cavalry, 165.0, 12.0, 4.2, 1.4, 3.3, [110.0, 50.0, 55.0], 16.0, 0.4, None, "弓と槍に長けた奴隷出身の最精鋭の騎兵"),
    ud!("スーダン歩兵", Infantry, 125.0, 13.0, 0.75, 1.2, 1.9, [55.0, 0.0, 25.0], 10.0, 1.0, None, "ファーティマ朝に仕えるヌビアの歩兵"),
    ud!("アルメニア弓兵", Archer, 80.0, 10.0, 4.7, 1.6, 1.75, [50.0, 35.0, 15.0], 11.0, 0.35, None, "ファーティマ朝の軍の中核をなしたアルメニア人の射手"),
    ud!("ガーズィー", Infantry, 115.0, 13.0, 0.75, 1.1, 2.0, [55.0, 0.0, 25.0], 10.0, 1.0, None, "信仰に燃える義勇の歩兵"),
    ud!("ナフサ兵", Archer, 80.0, 13.0, 3.0, 2.0, 1.7, [60.0, 30.0, 40.0], 12.0, 1.5, None, "燃える油を投げる。城と兵を焼く"),
    ud!("ベドウィン騎兵", Cavalry, 130.0, 13.0, 0.75, 1.3, 3.6, [70.0, 35.0, 30.0], 12.0, 0.6, None, "砂漠を駆けるアラブの軽騎兵"),
    ud!("クルド騎兵", Cavalry, 165.0, 15.0, 0.75, 1.4, 3.1, [90.0, 40.0, 50.0], 15.0, 0.7, None, "サラディンの一族の出たクルド人の騎兵"),
    ud!("ハルカ親衛騎兵", Cavalry, 180.0, 14.0, 4.2, 1.4, 3.2, [120.0, 50.0, 60.0], 17.0, 0.5, None, "スルタンの身辺を守るマムルークの親衛隊"),
    ud!("ムラービトの槍兵", Infantry, 130.0, 13.0, 0.75, 1.2, 1.9, [55.0, 0.0, 25.0], 10.0, 1.0, CAV, "革の盾を並べて戦うベルベル人の歩兵"),
];

pub const UT_WORKER: u8 = 0;
pub const UT_INF: u8 = 1;
pub const UT_ARC: u8 = 2;
pub const UT_CAV: u8 = 3;
pub const UT_HORSE_ARCHER: u8 = 4;
pub const UT_KNIGHT: u8 = 5;
pub const UT_SERGEANT: u8 = 6;
pub const UT_XBOW: u8 = 7;
pub const UT_PEOPLE: u8 = 8;
pub const UT_TEMPLAR: u8 = 9;
pub const UT_HOSPITALLER: u8 = 10;
pub const UT_TURCOPOLE: u8 = 11;
pub const UT_VARANGIAN: u8 = 12;
pub const UT_CATAPHRACT: u8 = 13;
pub const UT_DROMON: u8 = 14;
pub const UT_GALLEY: u8 = 15;
pub const UT_SHINI: u8 = 16;
pub const UT_GHULAM: u8 = 17;
pub const UT_MAMLUK: u8 = 18;
pub const UT_SUDANESE: u8 = 19;
pub const UT_ARMENIAN_ARCHER: u8 = 20;
pub const UT_GHAZI: u8 = 21;
pub const UT_NAPHTHA: u8 = 22;
pub const UT_BEDOUIN: u8 = 23;
pub const UT_KURD: u8 = 24;
pub const UT_HALQA: u8 = 25;
#[allow(dead_code)]
pub const UT_ALMORAVID: u8 = 26;

/// 国ごとに作れる兵（民を除く）。並びは NATIONS と同じ
pub const ROSTERS: &[&[u8]] = &[
    &[6, 7, 5, 11, 9, 10],   // 十字軍
    &[19, 20, 23, 17, 16],   // ファーティマ朝
    &[21, 2, 24, 18, 25, 22, 16], // アイユーブ朝
    &[12, 2, 11, 13, 14],    // ビザンツ帝国
    &[21, 2, 4, 17],         // ルーム・セルジューク朝
    &[21, 2, 4, 17],         // 大セルジューク朝
    &[1, 2, 4],              // ダニシュメンド朝
    &[21, 2, 4, 17],         // ダマスカス
    &[21, 2, 4, 17],         // アレッポ
    &[1, 4, 17],             // アルトゥク朝
    &[21, 2, 4, 18, 22],     // ザンギー朝
    &[21, 2, 17, 22],        // アッバース朝
    &[1, 2, 3, 11],          // アルメニア
    &[6, 7, 15],             // ヴェネツィア
    &[6, 7, 15],             // ジェノヴァ
    &[6, 7, 15],             // ピサ
    &[6, 7, 5],              // フランス王国
    &[6, 7, 5],              // 神聖ローマ帝国
    &[1, 2, 4, 5],           // ハンガリー王国
    &[6, 7, 5],              // ノルマン人
    &[6, 2, 5],              // イングランド王国
    &[6, 7, 5],              // カスティーリャ王国
    &[26, 2, 23],            // ムラービト朝
    &[1, 2, 23],             // ズィール朝
];

/// 船の定員
pub const SHIP_CAP: usize = 6;

pub fn udef(ut: u8) -> &'static UnitDef {
    UNIT_DEFS.get(ut as usize).unwrap_or(&UNIT_DEFS[0])
}

/// その国の基本の兵種（大分類に合うもの）
pub fn native_ut(n: usize, class: UnitKind) -> u8 {
    if class == UnitKind::Worker {
        return UT_WORKER;
    }
    if let Some(r) = ROSTERS.get(n) {
        if let Some(&u) = r.iter().find(|&&u| udef(u).class == class) {
            return u;
        }
    }
    match class {
        UnitKind::Infantry => UT_INF,
        UnitKind::Archer => UT_ARC,
        _ => UT_CAV,
    }
}

/// その国の最も強い陸兵（史実の出来事で馳せ参じる兵）
pub fn best_land_ut(n: usize) -> u8 {
    ROSTERS
        .get(n)
        .and_then(|r| r.iter().copied().filter(|&u| udef(u).class != UnitKind::Ship).max_by(|&a, &b| (udef(a).hp * udef(a).atk).partial_cmp(&(udef(b).hp * udef(b).atk)).unwrap()))
        .unwrap_or(UT_INF)
}

/// 船を造れる国か（自国の兵種に船がある）
pub fn native_ships(n: usize) -> bool {
    ROSTERS.get(n).map_or(false, |r| r.iter().any(|&u| udef(u).class == UnitKind::Ship))
}

impl World {
    /// 服属させた国（その国が築いた城邑を、いま自国が領している）
    pub fn subject_origins(&self, n: usize) -> Vec<usize> {
        let mut v: Vec<usize> = Vec::new();
        for b in self.blds.iter() {
            if b.alive && b.kind.city() && b.done() && b.nation as usize == n {
                let o = b.origin as usize;
                if o != n && o < NATIONS.len() && !v.contains(&o) {
                    v.push(o);
                }
            }
        }
        v.sort();
        v
    }

    /// その国が作れる兵種（自国の兵、続いて服属させた国の兵。重複なし）。(兵種, 出身国)
    pub fn available_units(&self, n: usize) -> Vec<(u8, usize)> {
        let mut v: Vec<(u8, usize)> = Vec::new();
        let push = |u: u8, o: usize, v: &mut Vec<(u8, usize)>| {
            if !v.iter().any(|x| x.0 == u) {
                v.push((u, o));
            }
        };
        if let Some(r) = ROSTERS.get(n) {
            for &u in r.iter() {
                push(u, n, &mut v);
            }
        }
        for o in self.subject_origins(n) {
            // 船は占領した城邑の国のものでも造れない
            for &u in ROSTERS[o].iter().filter(|&&u| udef(u).class != UnitKind::Ship) {
                push(u, o, &mut v);
            }
        }
        v
    }

    /// その国が船を造れるか（自国または服属させた国が船を持つ）
    pub fn can_build_ships(&self, n: usize) -> bool {
        n < NATIONS.len() && native_ships(n)
    }

    /// 建物で訓練できる兵種
    pub fn trainable_at(&self, b: &Building) -> Vec<(u8, usize)> {
        if !b.done() {
            return Vec::new();
        }
        let n = b.nation as usize;
        // 第1回十字軍の十字軍は兵も民も補充できない
        if self.no_reinforce(n) {
            return Vec::new();
        }
        match b.kind {
            BKind::Capital | BKind::Town => vec![(UT_WORKER, n)],
            BKind::Barracks => self.available_units(n).into_iter().filter(|&(u, _)| udef(u).class != UnitKind::Ship).collect(),
            BKind::Port => self.available_units(n).into_iter().filter(|&(u, _)| udef(u).class == UnitKind::Ship).collect(),
            BKind::Farm | BKind::Fort | BKind::Village => Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rosters_match_nations() {
        assert_eq!(ROSTERS.len(), NATIONS.len());
        for (n, r) in ROSTERS.iter().enumerate() {
            for &u in r.iter() {
                assert!((u as usize) < UNIT_DEFS.len());
                assert_ne!(u, UT_WORKER);
            }
            // 船を造れるのはイスラームの沿岸国家・イタリアの海の共和国・ビザンツ帝国だけ
            assert_eq!(native_ships(n), [FATIMID, AYYUBID, BYZANTIUM, VENICE, GENOA, PISA].contains(&n), "{}", NATIONS[n].name);
        }
    }
}
