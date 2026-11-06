//! 地理データ（西欧〜近東の海岸線・河川・山脈・砂漠）と国家・都市・遠征（シナリオ）のデータ、地形生成
//!
//! 地図の升目の数は3つの遠征で同じ（MAP_W×MAP_H）。遠征ごとに地図に写す範囲（経緯度）が違い、
//! 第1回は西欧から近東まで、第2回は中近東、第3回はレバント沿岸を大きく写す。

use crate::geodata::{LAKE_RINGS, LAND_RINGS, NE_RIVERS};
use crate::rng::{fbm, hash2, Rng};
use std::cell::Cell;

/// 地図の細かさの倍率（旧来の名残。1で固定）
pub const MAP_SCALE: usize = 1;
pub const MAP_W: usize = 600;
pub const MAP_H: usize = 340;

// ---------------------------------------------------------------- 遠征（シナリオ）
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Scen {
    /// 第1回十字軍（1096〜1099年）：西欧から近東まで
    First = 0,
    /// 第2回十字軍（1148年〜1250年）：中近東
    Second = 1,
    /// 第3回十字軍（1191〜1192年）：レバント沿岸
    Third = 2,
}

pub const SCENS: [Scen; 3] = [Scen::First, Scen::Second, Scen::Third];

impl Scen {
    pub fn from_u8(v: u8) -> Scen {
        SCENS[(v as usize).min(2)]
    }
    pub fn def(self) -> &'static ScenDef {
        &SCEN_DEFS[self as usize]
    }
    pub fn next(self) -> Scen {
        SCENS[(self as usize + 1) % 3]
    }
}

thread_local! {
    static CUR: Cell<Scen> = const { Cell::new(Scen::First) };
}

/// いまの遠征（地図の投影と都市の一覧を決める）
pub fn scen() -> Scen {
    CUR.with(|c| c.get())
}

/// 遠征を切り替える（地図を作る前・記録を読み込む前に呼ぶ）
pub fn set_scen(s: Scen) {
    CUR.with(|c| c.set(s));
}

fn proj() -> (f32, f32, f32, f32) {
    let d = scen().def();
    (d.lon0, d.lon1, d.lat_top, d.lat_bot)
}

pub fn ll_to_tile(lon: f32, lat: f32) -> (f32, f32) {
    let (l0, l1, lt, lb) = proj();
    ((lon - l0) / (l1 - l0) * MAP_W as f32, (lt - lat) / (lt - lb) * MAP_H as f32)
}
pub fn tile_to_ll(x: f32, y: f32) -> (f32, f32) {
    let (l0, l1, lt, lb) = proj();
    (l0 + x / MAP_W as f32 * (l1 - l0), lt - y / MAP_H as f32 * (lt - lb))
}
/// 1マスあたりの経度（度）
pub fn deg_per_tile() -> f32 {
    let (l0, l1, _, _) = proj();
    (l1 - l0) / MAP_W as f32
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Terrain {
    Sea,
    Lake,
    River,
    Plain,
    Steppe,
    Forest,
    Hill,
    Mountain,
    Desert,
}

impl Terrain {
    pub fn passable(self) -> bool {
        !matches!(self, Terrain::Sea | Terrain::Lake | Terrain::Mountain)
    }
    pub fn move_cost(self) -> f32 {
        match self {
            Terrain::Plain | Terrain::Steppe => 1.0,
            Terrain::Desert => 1.3,
            Terrain::Forest => 1.6,
            Terrain::Hill => 1.5,
            Terrain::River => 2.5,
            _ => f32::INFINITY,
        }
    }
    /// 船の移動コスト（海・湖・大河を進む）
    pub fn sail_cost(self) -> f32 {
        match self {
            Terrain::Sea | Terrain::Lake => 1.0,
            Terrain::River => 1.4,
            _ => f32::INFINITY,
        }
    }
    pub fn water(self) -> bool {
        matches!(self, Terrain::Sea | Terrain::Lake)
    }
    /// 建物を置ける地形
    pub fn buildable(self) -> bool {
        matches!(self, Terrain::Plain | Terrain::Steppe | Terrain::Hill | Terrain::Desert)
    }
}

// ---------------------------------------------------------------- 河川（手で補うもの）
struct River {
    pts: &'static [(f32, f32)],
    major: bool,
}

const RIVERS: &[River] = &[
    // オロンテス川（アシ川）：バールベックの北からホムス・ハマを経てアンティオキアへ
    River { major: false, pts: &[(36.35, 34.25), (36.55, 34.55), (36.70, 34.75), (36.75, 35.13), (36.55, 35.35), (36.40, 35.65), (36.32, 35.85), (36.30, 36.05), (36.16, 36.20), (35.95, 36.10), (35.92, 36.05)] },
    // リタニ川
    River { major: false, pts: &[(36.20, 33.95), (35.95, 33.70), (35.65, 33.40), (35.45, 33.33), (35.25, 33.34)] },
    // バラダ川（ダマスカスの川）
    River { major: false, pts: &[(36.05, 33.70), (36.18, 33.58), (36.29, 33.51), (36.45, 33.50)] },
    // ハブール川
    River { major: false, pts: &[(40.10, 36.85), (40.75, 36.50), (40.75, 35.90), (40.45, 35.15)] },
    // ザブ川
    River { major: false, pts: &[(44.40, 36.80), (43.80, 36.30), (43.35, 36.00)] },
    // ドナウ川の支流モラヴァ（ベオグラードからニシュへ）
    River { major: false, pts: &[(21.00, 44.70), (21.15, 44.20), (21.40, 43.80), (21.90, 43.40)] },
];

// 山脈（ポリライン, 幅 deg）。hill=true は丘陵までにとどめる（越えられる）
struct Range {
    pts: &'static [(f32, f32)],
    w: f32,
    hill: bool,
}
macro_rules! rg {
    ($w:expr, $p:expr) => {
        Range { w: $w, hill: false, pts: $p }
    };
    (hill $w:expr, $p:expr) => {
        Range { w: $w, hill: true, pts: $p }
    };
}
const RANGES: &[Range] = &[
    // ---- ヨーロッパ
    rg!(0.45, &[(5.6, 44.2), (6.9, 45.2), (7.6, 45.9), (9.0, 46.4), (10.5, 46.7), (12.5, 47.0), (14.0, 46.9), (15.6, 47.4)]), // アルプス
    rg!(0.30, &[(-1.8, 43.1), (-0.5, 42.8), (0.8, 42.7), (2.0, 42.5), (3.0, 42.4)]),                                     // ピレネー
    rg!(hill 0.6, &[(2.3, 45.0), (3.0, 45.6), (3.4, 44.6)]),                                                               // 中央山塊
    rg!(0.25, &[(9.0, 44.4), (10.5, 44.1), (12.0, 43.4), (13.3, 42.5), (14.5, 41.6), (15.5, 40.6), (16.0, 39.7)]),       // アペニン
    rg!(0.35, &[(18.6, 49.45), (21.0, 49.3), (23.0, 48.8), (24.5, 47.8), (25.6, 46.6), (25.5, 45.6), (23.5, 45.4), (22.0, 45.1)]), // カルパティア
    rg!(0.22, &[(22.6, 43.2), (24.0, 42.85), (25.0, 42.8), (26.8, 42.7)]),                                                // バルカン山脈
    rg!(0.20, &[(23.2, 42.1), (24.0, 41.7), (25.2, 41.5)]),                                                               // ロドピ
    rg!(0.30, &[(15.2, 45.0), (16.5, 44.0), (18.0, 43.2), (19.3, 42.5), (20.3, 41.6), (21.0, 40.2), (21.6, 39.2), (22.0, 38.4)]), // ディナル・ピンドス
    rg!(hill 0.6, &[(-6.0, 40.5), (-4.0, 40.5), (-2.0, 41.0)]),                                                            // イベリアの中央山系
    rg!(0.30, &[(-6.0, 31.5), (-4.0, 32.8), (-1.5, 34.0), (1.5, 35.0), (5.0, 35.6), (8.0, 35.6)]),                        // アトラス
    // ---- アナトリア・カフカス
    rg!(0.30, &[(29.5, 36.9), (30.6, 37.2), (32.5, 36.9), (34.0, 37.1), (34.9, 37.5), (35.6, 37.8), (37.0, 38.0), (39.0, 38.4), (41.0, 38.5), (42.5, 38.3)]), // タウルス
    rg!(0.22, &[(36.2, 36.3), (36.45, 36.75), (36.6, 37.2)]),                                                              // アマヌス
    rg!(hill 0.28, &[(31.5, 41.1), (33.5, 41.0), (36.0, 40.8), (39.0, 40.5), (41.5, 40.9)]),                               // ポントス山脈
    rg!(0.35, &[(37.6, 44.6), (40.0, 43.5), (42.5, 43.0), (44.0, 42.8), (45.3, 42.5), (46.6, 41.9), (47.8, 41.5)]),       // カフカス
    // ---- レバント
    rg!(0.10, &[(35.85, 33.45), (35.95, 33.75), (36.05, 34.05), (36.20, 34.40), (36.30, 34.60)]),                          // レバノン山脈
    rg!(0.09, &[(35.80, 33.35), (36.10, 33.62), (36.35, 33.90), (36.55, 34.20)]),                                          // アンチ・レバノン山脈
    rg!(0.07, &[(35.95, 35.20), (36.05, 35.55), (36.10, 35.85)]),                                                          // ヌサイリー山地
    rg!(hill 0.15, &[(35.10, 31.30), (35.15, 31.60), (35.20, 31.90), (35.25, 32.30)]),                                    // ユダヤの丘陵
    rg!(hill 0.10, &[(35.35, 32.70), (35.45, 33.05)]),                                                                     // ガリラヤの丘
    rg!(hill 0.12, &[(35.65, 30.20), (35.70, 30.80), (35.75, 31.30)]),                                                     // モアブ・エドムの山地
    rg!(hill 0.25, &[(33.6, 28.6), (34.0, 28.9), (34.3, 29.4)]),                                                           // シナイ山地
    // ---- メソポタミア・イラン
    rg!(0.45, &[(43.0, 37.4), (45.2, 37.0), (46.5, 35.0), (48.0, 33.0), (50.5, 30.5), (53.0, 28.5)]),                      // ザグロス
    rg!(0.30, &[(48.6, 37.4), (50.5, 36.4), (52.5, 35.9), (54.5, 36.6)]),                                                  // エルブルズ
    rg!(0.28, &[(36.0, 28.0), (37.2, 26.0)]),                                                                              // ヒジャーズ
];

/// 山越えの道（山脈を貫いて通れる幅を残す）
pub struct Road {
    pub pts: &'static [(f32, f32)],
}
pub const ROADS: &[Road] = &[
    // キリキアの門（タウルス越え）
    Road { pts: &[(34.70, 37.00), (34.80, 37.40), (34.90, 37.90)] },
    // シリアの門（ベレン峠：アマヌス越え）
    Road { pts: &[(36.10, 36.80), (36.25, 36.55), (36.30, 36.35), (36.16, 36.20)] },
    // ブレンナー峠（アルプス越え）
    Road { pts: &[(11.40, 47.30), (11.50, 47.00), (11.35, 46.50)] },
    // グラン・サン・ベルナール峠
    Road { pts: &[(7.20, 46.20), (7.17, 45.87), (7.30, 45.70)] },
    // 軍道（ベオグラード→ニシュ→ソフィア→フィリッポポリス→アドリアノープル→コンスタンティノープル）
    Road { pts: &[(20.46, 44.82), (21.90, 43.32), (23.32, 42.70), (24.75, 42.15), (26.55, 41.68), (27.80, 41.20), (28.97, 41.01)] },
    // エグナティア街道（デュラキウム→テッサロニキ）
    Road { pts: &[(19.45, 41.32), (20.20, 41.10), (20.80, 41.10), (21.90, 40.80), (22.95, 40.64)] },
    // ロンセスバーリェス（ピレネー越え）
    Road { pts: &[(-1.40, 43.30), (-1.30, 43.00), (-1.60, 42.80)] },
    // ベイルート→ダマスカス
    Road { pts: &[(35.50, 33.89), (35.80, 33.82), (36.00, 33.70), (36.29, 33.51)] },
    // トリポリ→ホムス（ホムスの谷）
    Road { pts: &[(35.84, 34.44), (36.20, 34.65), (36.72, 34.73)] },
    // ザグロス越え（バグダード→ハマダーン）
    Road { pts: &[(44.40, 33.30), (45.80, 34.20), (47.00, 34.40), (48.50, 34.80)] },
];

/// 海を陸に変える細い地峡（地図の目が粗く切れてしまう狭い海峡を、歩いて渡れるようにする）
const BRIDGES: &[&[(f32, f32)]] = &[
    // ボスポラス海峡（ビザンツの船が兵を渡す）
    &[(28.95, 40.98), (29.03, 41.02), (29.10, 41.08)],
];

fn dist_seg(px: f32, py: f32, a: (f32, f32), b: (f32, f32)) -> f32 {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let l2 = dx * dx + dy * dy;
    let t = if l2 > 0.0 { (((px - a.0) * dx + (py - a.1) * dy) / l2).clamp(0.0, 1.0) } else { 0.0 };
    let (cx, cy) = (a.0 + dx * t, a.1 + dy * t);
    ((px - cx).powi(2) + (py - cy).powi(2)).sqrt()
}

fn dist_poly(px: f32, py: f32, pts: &[(f32, f32)]) -> f32 {
    let mut d = f32::MAX;
    for w in pts.windows(2) {
        d = d.min(dist_seg(px, py, w[0], w[1]));
    }
    d
}

/// 輪の集まりを偶奇規則で塗る（行ごとに辺との交点を求める）
fn scan_fill(rings: &[&[(f32, f32)]], mut f: impl FnMut(i32, i32)) {
    let mut xs: Vec<f32> = Vec::new();
    for y in 0..MAP_H as i32 {
        let (_, lat) = tile_to_ll(0.0, y as f32 + 0.5);
        xs.clear();
        for r in rings {
            let n = r.len();
            for i in 0..n {
                let (x0, y0) = r[i];
                let (x1, y1) = r[(i + 1) % n];
                if (y0 > lat) != (y1 > lat) {
                    xs.push(x0 + (lat - y0) / (y1 - y0) * (x1 - x0));
                }
            }
        }
        xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
        for k in (0..xs.len().saturating_sub(1)).step_by(2) {
            let (a, _) = ll_to_tile(xs[k], lat);
            let (b, _) = ll_to_tile(xs[k + 1], lat);
            let a = (a - 0.5).ceil().max(0.0) as i32;
            let b = ((b - 0.5).floor() as i32).min(MAP_W as i32 - 1);
            for x in a..=b {
                f(x, y);
            }
        }
    }
}

/// 楕円の内側か（中心・半径は経緯度）
fn in_ell(lon: f32, lat: f32, c: (f32, f32, f32, f32)) -> bool {
    ((lon - c.0) / c.2).powi(2) + ((lat - c.1) / c.3).powi(2) < 1.0
}

/// 砂漠（中心経度・緯度・半径）
const DESERTS: &[(f32, f32, f32, f32)] = &[
    (39.0, 32.6, 3.4, 2.2),  // シリア砂漠
    (44.0, 27.0, 9.0, 3.6),  // アラビア（ネフド）
    (34.6, 30.3, 0.9, 0.9),  // ネゲヴ
    (33.6, 30.2, 1.1, 0.7),  // シナイ
    (55.0, 34.0, 3.0, 2.0),  // カヴィール
    (5.0, 28.0, 17.0, 3.2),  // サハラ
    (20.0, 29.5, 6.0, 1.9),  // リビア砂漠
];

/// ナイルの谷（砂漠にしない）
const NILE: &[(f32, f32)] = &[(32.9, 23.0), (32.9, 24.0), (32.7, 26.0), (31.7, 27.2), (30.9, 28.5), (31.2, 30.1)];

// ---------------------------------------------------------------- 国家データ
pub struct NationDef {
    pub name: &'static str,
    pub color: [f32; 3],
    /// 文化（絵柄）：4=イスラーム 5=ラテン 6=正教
    pub culture: u8,
    /// 大国
    pub major: bool,
    pub desc: &'static str,
}

macro_rules! nd {
    ($n:expr, $c:expr, $cu:expr, $s:expr, $desc:expr) => {
        NationDef { name: $n, color: $c, culture: $cu, major: $s, desc: $desc }
    };
}

pub const NATIONS: &[NationDef] = &[
    nd!("十字軍", [0.86, 0.16, 0.16], 5, true, "聖地を目指して東へ向かう西欧の騎士と民の連合軍"),
    nd!("ファーティマ朝", [0.18, 0.64, 0.34], 4, true, "カイロのイスマーイール派のカリフの国。ナイルの富と艦隊を持つ"),
    nd!("アイユーブ朝", [0.95, 0.76, 0.22], 4, true, "サラディンがエジプトとシリアをまとめて開いた王朝"),
    nd!("ビザンツ帝国", [0.56, 0.20, 0.60], 6, true, "コンスタンティノープルを都とするローマ帝国。ギリシアの火の艦隊を持つ"),
    nd!("ルーム・セルジューク朝", [0.20, 0.52, 0.56], 4, false, "アナトリアのトルコ人のスルタン国（セルジューク朝）"),
    nd!("大セルジューク朝", [0.16, 0.36, 0.30], 4, true, "イスファハーンのスルタンが東方を治めるトルコ人の大帝国"),
    nd!("ダニシュメンド朝", [0.58, 0.44, 0.26], 4, false, "アナトリア中部のトルコ人の君侯国"),
    nd!("ダマスカス", [0.72, 0.36, 0.62], 4, false, "ダマスカスのアミールの国（第1回はドゥカーク、第2回はブーリー朝）"),
    nd!("アレッポ", [0.86, 0.52, 0.24], 4, false, "アレッポのセルジューク家のリドワーンの国。アンティオキアを従える"),
    nd!("アルトゥク朝", [0.64, 0.62, 0.30], 4, false, "トルクメンのアルトゥク家の君侯国"),
    nd!("ザンギー朝", [0.30, 0.38, 0.76], 4, true, "モスルとアレッポのアタベク、ザンギー家の国。ヌールッディーンが聖戦を掲げる"),
    nd!("アッバース朝", [0.26, 0.26, 0.32], 4, false, "バグダードのカリフ。スンナ派の権威"),
    nd!("アルメニア", [0.90, 0.56, 0.36], 6, false, "キリキアとエデッサのアルメニア人の諸侯"),
    nd!("ヴェネツィア", [0.78, 0.22, 0.34], 5, false, "アドリア海の女王。地中海に商船と軍船を浮かべる海の共和国"),
    nd!("ジェノヴァ", [0.84, 0.84, 0.92], 5, false, "リグリアの海の共和国。十字軍に船を出した"),
    nd!("ピサ", [0.50, 0.16, 0.22], 5, false, "アルノ川の河口の海の共和国"),
    nd!("フランス王国", [0.26, 0.36, 0.82], 5, true, "カペー家のフランス王国"),
    nd!("神聖ローマ帝国", [0.90, 0.82, 0.36], 5, true, "ザーリアー家の皇帝が治めるドイツとイタリアの帝国"),
    nd!("ハンガリー王国", [0.40, 0.66, 0.30], 5, false, "アールパード家の王国。十字軍は国を通り抜けた"),
    nd!("ノルマン人", [0.30, 0.56, 0.86], 5, false, "南イタリアとシチリアを征したノルマン人の諸侯"),
    nd!("イングランド王国", [0.62, 0.18, 0.30], 5, false, "ノルマン朝のイングランド王国"),
    nd!("カスティーリャ王国", [0.80, 0.66, 0.30], 5, false, "トレドを奪ったレコンキスタの王国"),
    nd!("ムラービト朝", [0.46, 0.36, 0.24], 4, true, "マグリブとアンダルスを治めるベルベル人の王朝"),
    nd!("ズィール朝", [0.36, 0.56, 0.48], 4, false, "マフディーヤのイフリーキヤの王朝"),
];

pub const CRUSADERS: usize = 0;
pub const FATIMID: usize = 1;
pub const AYYUBID: usize = 2;
pub const BYZANTIUM: usize = 3;
pub const RUM: usize = 4;
pub const GREAT_SELJUK: usize = 5;
pub const DANISHMEND: usize = 6;
pub const DAMASCUS: usize = 7;
pub const ALEPPO: usize = 8;
pub const ARTUQID: usize = 9;
pub const ZENGID: usize = 10;
pub const ABBASID: usize = 11;
pub const ARMENIA: usize = 12;
pub const VENICE: usize = 13;
pub const GENOA: usize = 14;
pub const PISA: usize = 15;
pub const FRANCE: usize = 16;
pub const HRE: usize = 17;
pub const HUNGARY: usize = 18;
pub const NORMANS: usize = 19;
pub const ENGLAND: usize = 20;
pub const CASTILE: usize = 21;
pub const ALMORAVID: usize = 22;
pub const ZIRID: usize = 23;

/// イスラームの国か
pub fn muslim(n: usize) -> bool {
    NATIONS.get(n).map_or(false, |d| d.culture == 4)
}

// ---------------------------------------------------------------- 都市と遠征
/// 実在の都市（遠征の始まりの持ち主）
pub struct CityDef {
    pub name: &'static str,
    pub lon: f32,
    pub lat: f32,
    pub owner: usize,
    /// その国の都
    pub cap: bool,
    /// レバント沿岸の都市（第3回の勝敗に使う）
    pub coast: bool,
}

macro_rules! cd {
    ($n:expr, $lon:expr, $lat:expr, $o:expr) => {
        CityDef { name: $n, lon: $lon, lat: $lat, owner: $o, cap: false, coast: false }
    };
    (cap $n:expr, $lon:expr, $lat:expr, $o:expr) => {
        CityDef { name: $n, lon: $lon, lat: $lat, owner: $o, cap: true, coast: false }
    };
    (coast $n:expr, $lon:expr, $lat:expr, $o:expr) => {
        CityDef { name: $n, lon: $lon, lat: $lat, owner: $o, cap: false, coast: true }
    };
    (cap coast $n:expr, $lon:expr, $lat:expr, $o:expr) => {
        CityDef { name: $n, lon: $lon, lat: $lat, owner: $o, cap: true, coast: true }
    };
}

/// 選べる国（説明・難易度 0:易 1:中 2:難）
pub struct Pick {
    pub nation: usize,
    pub desc: &'static str,
    pub diff: u8,
}

pub struct ScenDef {
    pub name: &'static str,
    pub years: &'static str,
    pub intro: &'static str,
    pub start_ad: f32,
    pub end_ad: f32,
    /// 1年の長さ（秒）
    pub spy: f32,
    pub lon0: f32,
    pub lon1: f32,
    pub lat_top: f32,
    pub lat_bot: f32,
    pub picks: &'static [Pick],
    /// 登場する国と生産力
    pub present: &'static [(usize, f32)],
    pub cities: &'static [CityDef],
    /// 始まりの時点で交戦している国
    pub wars: &'static [(usize, usize)],
    /// 都市のまわりの民家の数
    pub villages: u32,
    /// 始まりから兵営のある都市（都にはいつもある）
    pub barracks: &'static [&'static str],
    /// 始まりから城壁の要塞がある都市（都市, 数）
    pub forts: &'static [(&'static str, u32)],
}

const C1: &[CityDef] = &[
    // 西欧
    cd!(cap "パリ", 2.35, 48.86, FRANCE),
    cd!("オルレアン", 1.90, 47.90, FRANCE),
    cd!("ルーアン", 1.10, 49.44, ENGLAND),
    cd!("クレルモン", 3.08, 45.78, FRANCE),
    cd!("トゥールーズ", 1.44, 43.60, FRANCE),
    cd!("マルセイユ", 5.37, 43.30, FRANCE),
    cd!(cap "ロンドン", -0.13, 51.51, ENGLAND),
    cd!("ウィンチェスター", -1.31, 51.06, ENGLAND),
    cd!(cap "トレド", -4.02, 39.86, CASTILE),
    cd!("レオン", -5.57, 42.60, CASTILE),
    cd!(cap "セビーリャ", -5.98, 37.39, ALMORAVID),
    cd!("コルドバ", -4.78, 37.88, ALMORAVID),
    cd!("サラゴサ", -0.89, 41.65, ALMORAVID),
    cd!(cap "マフディーヤ", 11.06, 35.50, ZIRID),
    cd!("チュニス", 10.18, 36.80, ZIRID),
    cd!(cap "マインツ", 8.27, 50.00, HRE),
    cd!("ケルン", 6.96, 50.94, HRE),
    cd!("レーゲンスブルク", 12.10, 49.02, HRE),
    cd!("ウィーン", 16.37, 48.21, HRE),
    cd!("ミラノ", 9.19, 45.46, HRE),
    cd!("ローマ", 12.48, 41.89, HRE),
    cd!(cap "ヴェネツィア", 12.33, 45.44, VENICE),
    cd!("ザラ", 15.23, 44.12, VENICE),
    cd!(cap "ジェノヴァ", 8.93, 44.41, GENOA),
    cd!(cap "ピサ", 10.40, 43.72, PISA),
    cd!(cap "パレルモ", 13.36, 38.12, NORMANS),
    cd!("バーリ", 16.87, 41.12, NORMANS),
    cd!("ナポリ", 14.27, 40.85, NORMANS),
    cd!("サレルノ", 14.77, 40.68, NORMANS),
    cd!(cap "エステルゴム", 18.74, 47.79, HUNGARY),
    cd!("セーケシュフェヘールヴァール", 18.41, 47.19, HUNGARY),
    cd!("ナンドルフェヘールヴァール", 20.46, 44.82, HUNGARY),
    // ビザンツ帝国
    cd!(cap "コンスタンティノープル", 28.97, 41.01, BYZANTIUM),
    cd!("テッサロニキ", 22.95, 40.64, BYZANTIUM),
    cd!("デュラキウム", 19.45, 41.32, BYZANTIUM),
    cd!("アドリアノープル", 26.55, 41.68, BYZANTIUM),
    cd!("フィリッポポリス", 24.75, 42.15, BYZANTIUM),
    cd!("ソフィア", 23.32, 42.70, BYZANTIUM),
    cd!("ニシュ", 21.90, 43.32, BYZANTIUM),
    cd!("アテネ", 23.73, 37.98, BYZANTIUM),
    cd!("ニコメディア", 29.92, 40.77, BYZANTIUM),
    cd!("アッタレイア", 30.70, 36.89, BYZANTIUM),
    cd!("トレビゾンド", 39.72, 41.00, BYZANTIUM),
    cd!("カンディア", 25.13, 35.34, BYZANTIUM),
    cd!("ニコシア", 33.36, 35.17, BYZANTIUM),
    // アナトリア
    cd!(cap "ニカイア", 29.72, 40.43, RUM),
    cd!("ドリュラエウム", 30.52, 39.78, RUM),
    cd!("イコニオン", 32.48, 37.87, RUM),
    cd!("スミルナ", 27.14, 38.42, RUM),
    cd!("ヘラクレイア", 34.05, 37.51, RUM),
    cd!(cap "シヴァス", 37.02, 39.75, DANISHMEND),
    cd!("アンキュラ", 32.85, 39.93, DANISHMEND),
    cd!("カイセリ", 35.49, 38.72, DANISHMEND),
    cd!(cap "エデッサ", 38.79, 37.16, ARMENIA),
    cd!("メリテネ", 38.31, 38.35, ARMENIA),
    cd!("タルソス", 34.89, 36.92, ARMENIA),
    cd!("マラシュ", 36.93, 37.58, ARMENIA),
    // シリア・メソポタミア
    cd!(cap "アレッポ", 37.16, 36.20, ALEPPO),
    cd!("アンティオキア", 36.16, 36.20, ALEPPO),
    cd!("ハマ", 36.75, 35.13, ALEPPO),
    cd!(cap "ダマスカス", 36.29, 33.51, DAMASCUS),
    cd!("ホムス", 36.72, 34.73, DAMASCUS),
    cd!("バールベック", 36.21, 34.00, DAMASCUS),
    cd!(cap "マルディン", 40.74, 37.31, ARTUQID),
    cd!("エルサレム", 35.23, 31.78, ARTUQID),
    cd!(cap "イスファハーン", 51.67, 32.65, GREAT_SELJUK),
    cd!("バグダード", 44.40, 33.31, GREAT_SELJUK),
    cd!("モスル", 43.13, 36.34, GREAT_SELJUK),
    cd!("ハマダーン", 48.51, 34.80, GREAT_SELJUK),
    cd!("ラッカ", 39.01, 35.95, GREAT_SELJUK),
    // ファーティマ朝
    cd!(cap "カイロ", 31.24, 30.04, FATIMID),
    cd!("アレクサンドリア", 29.92, 31.20, FATIMID),
    cd!("ダミエッタ", 31.81, 31.42, FATIMID),
    cd!("アスカロン", 34.55, 31.66, FATIMID),
    cd!("ヤッファ", 34.75, 32.05, FATIMID),
    cd!("アッコン", 35.07, 32.93, FATIMID),
    cd!("ティルス", 35.20, 33.27, FATIMID),
    cd!("シドン", 35.37, 33.56, FATIMID),
    cd!("トリポリ", 35.84, 34.44, FATIMID),
];

const C2: &[CityDef] = &[
    // 十字軍国家（エルサレム王国・アンティオキア公国・トリポリ伯国）
    cd!(cap "エルサレム", 35.23, 31.78, CRUSADERS),
    cd!("アッコン", 35.07, 32.93, CRUSADERS),
    cd!("ティルス", 35.20, 33.27, CRUSADERS),
    cd!("ヤッファ", 34.75, 32.05, CRUSADERS),
    cd!("ガザ", 34.46, 31.50, CRUSADERS),
    cd!("シドン", 35.37, 33.56, CRUSADERS),
    cd!("ベイルート", 35.50, 33.89, CRUSADERS),
    cd!("ケラク", 35.70, 31.18, CRUSADERS),
    cd!("ティベリアス", 35.53, 32.79, CRUSADERS),
    cd!("トリポリ", 35.84, 34.44, CRUSADERS),
    cd!("トルトサ", 35.89, 34.89, CRUSADERS),
    cd!("アンティオキア", 36.16, 36.20, CRUSADERS),
    cd!("ラタキア", 35.78, 35.52, CRUSADERS),
    // ファーティマ朝
    cd!(cap "カイロ", 31.24, 30.04, FATIMID),
    cd!("アレクサンドリア", 29.92, 31.20, FATIMID),
    cd!("ダミエッタ", 31.81, 31.42, FATIMID),
    cd!("アスカロン", 34.55, 31.66, FATIMID),
    // ダマスカス（ブーリー朝）
    cd!(cap "ダマスカス", 36.29, 33.51, DAMASCUS),
    cd!("バールベック", 36.21, 34.00, DAMASCUS),
    cd!("ボスラ", 36.48, 32.52, DAMASCUS),
    cd!("ホムス", 36.72, 34.73, DAMASCUS),
    // ザンギー朝
    cd!(cap "アレッポ", 37.16, 36.20, ZENGID),
    cd!("モスル", 43.13, 36.34, ZENGID),
    cd!("エデッサ", 38.79, 37.16, ZENGID),
    cd!("ハマ", 36.75, 35.13, ZENGID),
    cd!("ハッラーン", 39.03, 36.86, ZENGID),
    cd!("ラッカ", 39.01, 35.95, ZENGID),
    // アルトゥク朝
    cd!(cap "マルディン", 40.74, 37.31, ARTUQID),
    cd!("アーミド", 40.23, 37.91, ARTUQID),
    // アッバース朝
    cd!(cap "バグダード", 44.40, 33.31, ABBASID),
    cd!("ワーシト", 46.10, 32.20, ABBASID),
    cd!("バスラ", 47.80, 30.50, ABBASID),
    // ルーム・セルジューク朝
    cd!(cap "イコニオン", 32.48, 37.87, RUM),
    cd!("カイセリ", 35.49, 38.72, RUM),
    cd!("アンキュラ", 32.85, 39.93, RUM),
    cd!("シヴァス", 37.02, 39.75, RUM),
    cd!("メリテネ", 38.31, 38.35, RUM),
    // キリキア・アルメニア
    cd!(cap "シス", 35.82, 37.45, ARMENIA),
    cd!("マラシュ", 36.93, 37.58, ARMENIA),
    // ビザンツ帝国
    cd!(cap "コンスタンティノープル", 28.97, 41.01, BYZANTIUM),
    cd!("ニカイア", 29.72, 40.43, BYZANTIUM),
    cd!("ニコメディア", 29.92, 40.77, BYZANTIUM),
    cd!("スミルナ", 27.14, 38.42, BYZANTIUM),
    cd!("アッタレイア", 30.70, 36.89, BYZANTIUM),
    cd!("ドリュラエウム", 30.52, 39.78, BYZANTIUM),
    cd!("タルソス", 34.89, 36.92, BYZANTIUM),
    cd!("トレビゾンド", 39.72, 41.00, BYZANTIUM),
    cd!("ニコシア", 33.36, 35.17, BYZANTIUM),
    cd!("ロドス", 28.22, 36.43, BYZANTIUM),
    cd!("カンディア", 25.13, 35.34, BYZANTIUM),
];

const C3: &[CityDef] = &[
    // 十字軍連合軍（アッコン包囲の陣・ティルス・トリポリ・アンティオキア・キプロス）
    cd!(coast "ティルス", 35.20, 33.27, CRUSADERS),
    cd!(cap coast "トリポリ", 35.84, 34.44, CRUSADERS),
    cd!(coast "トルトサ", 35.89, 34.89, CRUSADERS),
    cd!("アンティオキア", 36.16, 36.20, CRUSADERS),
    cd!("マルガット", 35.95, 35.15, CRUSADERS),
    cd!("リマソル", 33.04, 34.68, CRUSADERS),
    cd!("ニコシア", 33.36, 35.17, CRUSADERS),
    cd!("ファマグスタ", 33.94, 35.12, CRUSADERS),
    // アイユーブ朝
    cd!(cap "カイロ", 31.24, 30.04, AYYUBID),
    cd!("アレクサンドリア", 29.92, 31.20, AYYUBID),
    cd!("ダミエッタ", 31.81, 31.42, AYYUBID),
    cd!(coast "アッコン", 35.07, 32.93, AYYUBID),
    cd!(coast "ハイファ", 34.99, 32.80, AYYUBID),
    cd!(coast "カエサレア", 34.89, 32.50, AYYUBID),
    cd!(coast "アルスーフ", 34.81, 32.20, AYYUBID),
    cd!(coast "ヤッファ", 34.75, 32.05, AYYUBID),
    cd!(coast "アスカロン", 34.55, 31.66, AYYUBID),
    cd!(coast "ガザ", 34.46, 31.50, AYYUBID),
    cd!(coast "ダルム", 34.32, 31.38, AYYUBID),
    cd!(coast "シドン", 35.37, 33.56, AYYUBID),
    cd!(coast "ベイルート", 35.50, 33.89, AYYUBID),
    cd!(coast "ジュバイル", 35.65, 34.12, AYYUBID),
    cd!(coast "ジャブラ", 35.93, 35.36, AYYUBID),
    cd!(coast "ラタキア", 35.78, 35.52, AYYUBID),
    cd!("エルサレム", 35.23, 31.78, AYYUBID),
    cd!("ラムラ", 34.87, 31.93, AYYUBID),
    cd!("ナーブルス", 35.26, 32.22, AYYUBID),
    cd!("ティベリアス", 35.53, 32.79, AYYUBID),
    cd!("サフェド", 35.50, 32.96, AYYUBID),
    cd!("ケラク", 35.70, 31.18, AYYUBID),
    cd!("ダマスカス", 36.29, 33.51, AYYUBID),
    cd!("バールベック", 36.21, 34.00, AYYUBID),
    cd!("ホムス", 36.72, 34.73, AYYUBID),
    cd!("ハマ", 36.75, 35.13, AYYUBID),
    cd!("アレッポ", 37.16, 36.20, AYYUBID),
    cd!("ボスラ", 36.48, 32.52, AYYUBID),
];

pub const SCEN_DEFS: &[ScenDef] = &[
    ScenDef {
        name: "第1回十字軍",
        years: "1096 — 1099",
        intro: "1095年、教皇ウルバヌス2世はクレルモンで聖地の回復を呼びかけた。「神がそれを望みたもう！」。翌年、西欧の諸侯と民が東へ発つ。十字軍は出発の時が最大の兵力で、補充はできない。進軍で疲れ、占領地や民家の襲撃で力を取り戻す。\n\n十字軍：史実の占領（1099年7月）までにエルサレムを奪い、5年守れば勝利。\nファーティマ朝：それを阻むか、十字軍を全滅させれば勝利。",
        start_ad: 1096.6,
        end_ad: 1104.55,
        spy: 240.0,
        lon0: -6.0,
        lon1: 54.0,
        lat_top: 54.0,
        lat_bot: 27.0,
        picks: &[
            Pick { nation: CRUSADERS, desc: "ゴドフロワ・ボエモン・レーモンら諸侯と隠者ピエールの民衆十字軍。都市を持たずに西欧の各地から発ち、兵は補充できない。ビザンツ領では騎士だけが傷を癒せる", diff: 1 },
            Pick { nation: FATIMID, desc: "宰相アル＝アフダルが治めるエジプトのカリフ国。セルジューク諸国と争いながら、十字軍からエルサレムを守り抜け", diff: 1 },
        ],
        present: &[
            (CRUSADERS, 1.0), (FATIMID, 1.35), (BYZANTIUM, 1.4), (RUM, 1.0), (GREAT_SELJUK, 1.2), (DANISHMEND, 0.9), (DAMASCUS, 0.95),
            (ALEPPO, 0.95), (ARTUQID, 0.9), (ARMENIA, 0.8), (VENICE, 1.0), (GENOA, 0.9), (PISA, 0.9), (FRANCE, 1.0), (HRE, 1.1),
            (HUNGARY, 1.0), (NORMANS, 1.0), (ENGLAND, 0.95), (CASTILE, 0.95), (ALMORAVID, 1.1), (ZIRID, 0.9),
        ],
        cities: C1,
        wars: &[
            (CRUSADERS, FATIMID), (CRUSADERS, RUM), (CRUSADERS, GREAT_SELJUK), (CRUSADERS, DANISHMEND), (CRUSADERS, DAMASCUS),
            (CRUSADERS, ALEPPO), (CRUSADERS, ARTUQID), (CASTILE, ALMORAVID),
        ],
        villages: 2,
        barracks: &[],
        forts: &[("コンスタンティノープル", 2), ("ニカイア", 2), ("アンティオキア", 2), ("エルサレム", 2), ("アレッポ", 1), ("ダマスカス", 1), ("カイロ", 1)],
    },
    ScenDef {
        name: "第2回十字軍",
        years: "1148 — 1250",
        intro: "1144年、ザンギーがエデッサを陥れた。聖ベルナールの呼びかけに、フランス王ルイ7世とドイツ王コンラート3世が十字を負い、1148年に聖地へ上陸する。\n\n十字軍：全滅せずにダマスカスを占領すれば勝利。\nほかの国：1250年に最も広い版図を持つ国が勝利。",
        start_ad: 1148.4,
        end_ad: 1250.0,
        spy: 24.0,
        lon0: 23.5,
        lon1: 52.0,
        lat_top: 41.8,
        lat_bot: 28.6,
        picks: &[
            Pick { nation: CRUSADERS, desc: "上陸したルイ7世とコンラート3世の軍と、エルサレム王国・アンティオキア公国・トリポリ伯国。全滅せずにダマスカスを奪え", diff: 1 },
            Pick { nation: FATIMID, desc: "衰えゆくカイロのカリフ国。ナイルの富と艦隊で1250年まで版図を競う", diff: 2 },
            Pick { nation: ZENGID, desc: "ヌールッディーンのアレッポとモスルの国。聖戦を掲げてシリアの統一を目指す", diff: 0 },
            Pick { nation: RUM, desc: "コンヤのスルタン・マスウード。ドリュラエウムでドイツ軍を破ったアナトリアのトルコ人の国", diff: 1 },
            Pick { nation: BYZANTIUM, desc: "皇帝マヌエル1世コムネノス。コムネノス朝の栄光を取り戻し、東方に覇を唱える", diff: 0 },
        ],
        present: &[(CRUSADERS, 1.1), (FATIMID, 1.15), (ZENGID, 1.25), (RUM, 1.1), (BYZANTIUM, 1.35), (DAMASCUS, 0.95), (ARTUQID, 0.85), (ABBASID, 0.9), (ARMENIA, 0.85)],
        cities: C2,
        wars: &[(CRUSADERS, DAMASCUS), (CRUSADERS, ZENGID), (CRUSADERS, FATIMID), (RUM, BYZANTIUM)],
        villages: 2,
        barracks: &["アッコン", "アンティオキア", "トリポリ", "モスル", "エデッサ", "アスカロン", "ニカイア", "カイセリ"],
        forts: &[("ダマスカス", 3), ("アレッポ", 2), ("コンスタンティノープル", 2), ("カイロ", 1), ("エルサレム", 2), ("アンティオキア", 1), ("アッコン", 1), ("イコニオン", 1), ("バグダード", 1), ("モスル", 1)],
    },
    ScenDef {
        name: "第3回十字軍",
        years: "1191 — 1192",
        intro: "1187年、サラディンはヒッティーンで十字軍を破り、エルサレムを取り戻した。リチャード獅子心王とフィリップ2世がアッコンの包囲の陣に上陸する。\n\n1192年までに、レバント沿岸の都市から相手を駆逐した側の勝利。駆逐できなければ和約を結んで終わる。",
        start_ad: 1191.43,
        end_ad: 1192.67,
        spy: 1200.0,
        lon0: 27.3,
        lon1: 43.8,
        lat_top: 37.2,
        lat_bot: 29.4,
        picks: &[
            Pick { nation: CRUSADERS, desc: "リチャード獅子心王・フィリップ2世・ギー・ド・リュジニャン・コンラートの連合軍。アッコンを落とし、沿岸の都市をすべて取り戻せ", diff: 1 },
            Pick { nation: AYYUBID, desc: "スルタン・サラディンとアル＝アーディル。アッコンを救い、ティルスとトリポリから十字軍を海へ追い落とせ", diff: 1 },
        ],
        present: &[(CRUSADERS, 1.25), (AYYUBID, 1.15)],
        cities: C3,
        wars: &[(CRUSADERS, AYYUBID)],
        villages: 2,
        barracks: &["ティルス", "アンティオキア", "ダマスカス", "エルサレム", "アスカロン", "アレッポ", "ナーブルス"],
        forts: &[("アッコン", 2), ("ティルス", 2), ("トリポリ", 1), ("エルサレム", 1), ("ダマスカス", 1), ("アスカロン", 1)],
    },
];

/// いまの遠征の都市
pub fn cities() -> &'static [CityDef] {
    scen().def().cities
}

/// 遠征に登場する国か
pub fn present(s: Scen, n: usize) -> bool {
    s.def().present.iter().any(|&(k, _)| k == n)
}

/// 遠征での国の生産力（登場しない国は0）
pub fn bias_of(s: Scen, n: usize) -> f32 {
    s.def().present.iter().find(|&&(k, _)| k == n).map_or(0.0, |&(_, b)| b)
}

/// 都市の名前から番号
pub fn city_index(name: &str) -> Option<usize> {
    cities().iter().position(|c| c.name == name)
}

// ---------------------------------------------------------------- 地図
pub struct Mine {
    pub x: i32,
    pub y: i32,
    pub amount: f32,
}

pub struct Map {
    pub terrain: Vec<Terrain>,
    pub wood: Vec<f32>,
    pub shade: Vec<f32>,
    pub mines: Vec<Mine>,
}

impl Map {
    #[inline]
    pub fn idx(x: i32, y: i32) -> usize {
        y as usize * MAP_W + x as usize
    }
    #[inline]
    pub fn inb(x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < MAP_W as i32 && y < MAP_H as i32
    }
    pub fn t(&self, x: i32, y: i32) -> Terrain {
        if Self::inb(x, y) {
            self.terrain[Self::idx(x, y)]
        } else {
            Terrain::Mountain
        }
    }
}

fn raster_line(map: &mut Map, a: (f32, f32), b: (f32, f32), thick: bool) {
    let (ax, ay) = ll_to_tile(a.0, a.1);
    let (bx, by) = ll_to_tile(b.0, b.1);
    let len = ((bx - ax).powi(2) + (by - ay).powi(2)).sqrt();
    let steps = (len * 4.0).ceil().max(1.0) as i32;
    for s in 0..=steps {
        let t = s as f32 / steps as f32;
        let x = (ax + (bx - ax) * t).floor() as i32;
        let y = (ay + (by - ay) * t).floor() as i32;
        let pts: &[(i32, i32)] = if thick { &[(0, 0), (1, 0), (0, 1)] } else { &[(0, 0)] };
        for &(dx, dy) in pts {
            let (xx, yy) = (x + dx, y + dy);
            if Map::inb(xx, yy) {
                let i = Map::idx(xx, yy);
                if map.terrain[i] != Terrain::Sea && map.terrain[i] != Terrain::Lake {
                    map.terrain[i] = Terrain::River;
                }
            }
        }
    }
}

/// 線に沿ってマスを書き換える（f が真を返したマスだけ）
fn along_line(pts: &[(f32, f32)], width: i32, mut f: impl FnMut(usize)) {
    for w in pts.windows(2) {
        let (ax, ay) = ll_to_tile(w[0].0, w[0].1);
        let (bx, by) = ll_to_tile(w[1].0, w[1].1);
        let len = ((bx - ax).powi(2) + (by - ay).powi(2)).sqrt();
        let steps = (len * 3.0).ceil().max(1.0) as i32;
        for s in 0..=steps {
            let t = s as f32 / steps as f32;
            let x = (ax + (bx - ax) * t).floor() as i32;
            let y = (ay + (by - ay) * t).floor() as i32;
            for dy in -width..=width {
                for dx in -width..=width {
                    if Map::inb(x + dx, y + dy) {
                        f(Map::idx(x + dx, y + dy));
                    }
                }
            }
        }
    }
}

pub fn generate_map(seed: u32) -> Map {
    let n = MAP_W * MAP_H;
    let mut map = Map { terrain: vec![Terrain::Sea; n], wood: vec![0.0; n], shade: vec![0.0; n], mines: Vec::new() };
    // 陸地
    let mut land = vec![false; n];
    scan_fill(LAND_RINGS, |x, y| land[Map::idx(x, y)] = true);
    // 地図の縮尺（1マスあたりの度）。揺らぎや山脈の幅をマスの単位でそろえる
    let dpt = deg_per_tile();
    let (_, lt, _, lb) = { let p = proj(); (p.0, p.2, p.1, p.3) };
    let dpt_lat = (lt - lb) / MAP_H as f32;

    for y in 0..MAP_H as i32 {
        for x in 0..MAP_W as i32 {
            let (lon, lat) = tile_to_ll(x as f32 + 0.5, y as f32 + 0.5);
            let i = Map::idx(x, y);
            let nz = fbm(x as f32 * 0.08, y as f32 * 0.08, seed, 4);
            let nz2 = fbm(x as f32 * 0.15, y as f32 * 0.15, seed ^ 0x55aa, 3);
            map.shade[i] = nz;
            if !land[i] {
                map.terrain[i] = Terrain::Sea;
                continue;
            }
            let mut t = Terrain::Plain;
            let wlon = lon + ((nz - 0.5) * 12.0 + (nz2 - 0.5) * 5.5) * dpt;
            let wlat = lat + ((nz2 - 0.5) * 9.5 + (nz - 0.5) * 5.5) * dpt_lat;
            // 草原（アナトリア高原・パンノニア平原・黒海の北・ジャズィーラ・イラン高原）
            let steppe = in_ell(wlon, wlat, (33.5, 38.9, 3.2, 1.2))
                || in_ell(wlon, wlat, (20.0, 46.8, 2.0, 1.1))
                || (wlat > 45.5 && wlon > 29.0)
                || in_ell(wlon, wlat, (40.0, 36.2, 2.6, 0.9))
                || in_ell(wlon, wlat, (52.0, 34.5, 3.0, 2.0))
                || in_ell(wlon, wlat, (37.6, 34.6, 0.9, 1.0));
            if steppe {
                t = Terrain::Steppe;
            }
            // 砂漠
            let mut desert = DESERTS.iter().any(|&d| in_ell(wlon, wlat, d));
            // エジプトの砂漠（ナイルの谷とデルタを除く）
            if wlat < 31.0 && wlon > 24.0 && wlon < 34.8 && (wlon < 33.6 || wlat < 30.0) && dist_poly(lon, lat, NILE) > 0.35 && !in_ell(lon, lat, (31.0, 30.9, 1.25, 0.75)) {
                desert = true;
            }
            // 北アフリカの内陸
            if wlat < 30.8 && wlon < 24.0 {
                desert = true;
            }
            if desert {
                t = Terrain::Desert;
            }
            // 丘陵の多い土地：アナトリア、バルカン、イタリア、イベリア、イラン、レバノン
            let hilly = in_ell(wlon, wlat, (35.0, 38.8, 7.0, 2.4))
                || in_ell(wlon, wlat, (21.5, 42.0, 4.0, 2.5))
                || in_ell(wlon, wlat, (14.0, 41.5, 2.0, 2.5))
                || in_ell(wlon, wlat, (-4.0, 40.0, 3.0, 2.0))
                || in_ell(wlon, wlat, (48.5, 35.5, 4.5, 2.5))
                || in_ell(wlon, wlat, (43.0, 39.5, 3.5, 1.5))
                || in_ell(wlon, wlat, (36.0, 33.8, 0.5, 0.7));
            if hilly && t != Terrain::Desert {
                if nz2 > 0.45 {
                    t = Terrain::Hill;
                }
                if nz2 > 0.70 {
                    t = Terrain::Mountain;
                }
            }
            // 山脈（細くても1マスは残す）
            for r in RANGES {
                let d = dist_poly(lon, lat, r.pts);
                let w = r.w.max(dpt * 1.6);
                let jitter = (nz2 - 0.5) * w * 0.8;
                if d < w * 0.55 + jitter {
                    t = if r.hill { Terrain::Hill } else { Terrain::Mountain };
                    if !r.hill {
                        break;
                    }
                } else if d < w * 1.5 + jitter && t != Terrain::Mountain {
                    t = Terrain::Hill;
                }
            }
            // 森林（西欧の森、レバノンの杉、近東には少ない）
            if matches!(t, Terrain::Plain | Terrain::Steppe) {
                let fthr = if lat > 44.0 && lon < 30.0 {
                    0.52
                } else if t == Terrain::Steppe {
                    0.80
                } else if lat < 37.5 && lon > 30.0 {
                    0.80
                } else if lat < 36.0 {
                    0.84
                } else {
                    0.63
                };
                if nz > fthr {
                    t = Terrain::Forest;
                }
            }
            map.terrain[i] = t;
        }
    }
    // 湖
    scan_fill(LAKE_RINGS, |x, y| map.terrain[Map::idx(x, y)] = Terrain::Lake);
    // 河川（大河は地図が細かいほど太く）
    let thick = dpt < 0.05;
    for &(major, pts) in NE_RIVERS {
        for w in pts.windows(2) {
            raster_line(&mut map, w[0], w[1], major || thick);
        }
    }
    for r in RIVERS {
        for w in r.pts.windows(2) {
            raster_line(&mut map, w[0], w[1], r.major || thick);
        }
    }
    // 狭い海峡を陸でつなぐ
    for b in BRIDGES {
        let t = &mut map.terrain;
        along_line(b, if thick { 1 } else { 0 }, |i| {
            if t[i] == Terrain::Sea {
                t[i] = Terrain::Hill;
            }
        });
    }
    // 山越えの道：山を丘に削り、通れるようにする
    for r in ROADS {
        let t = &mut map.terrain;
        along_line(r.pts, if thick { 1 } else { 0 }, |i| {
            if matches!(t[i], Terrain::Mountain | Terrain::Forest) {
                t[i] = Terrain::Hill;
            }
        });
    }
    // 都市の周囲を平地にし、木材・鉄を保証
    let mut rng = Rng::new(seed as u64 ^ 0x1234);
    for c in cities() {
        let (cx, cy) = ll_to_tile(c.lon, c.lat);
        let (cx, cy) = (cx as i32, cy as i32);
        for dy in -3..=4i32 {
            for dx in -3..=4i32 {
                let (x, y) = (cx + dx, cy + dy);
                if Map::inb(x, y) && dx * dx + dy * dy <= 13 {
                    let i = Map::idx(x, y);
                    if !matches!(map.terrain[i], Terrain::Sea | Terrain::River | Terrain::Lake) || (dx.abs() <= 1 && dy.abs() <= 1 && map.terrain[i] != Terrain::Sea) {
                        map.terrain[i] = Terrain::Plain;
                    }
                }
            }
        }
        ensure_forest(&mut map, cx, cy, &mut rng);
    }
    for i in 0..n {
        if map.terrain[i] == Terrain::Forest {
            map.wood[i] = 120.0;
        }
    }
    // 鉱山
    for y in 2..MAP_H as i32 - 2 {
        for x in 2..MAP_W as i32 - 2 {
            if map.terrain[Map::idx(x, y)] == Terrain::Hill && hash2(x, y, seed ^ 0x77) < 0.035 {
                if map.mines.iter().all(|m| (m.x - x).abs() + (m.y - y).abs() > 10) {
                    map.mines.push(Mine { x, y, amount: 2500.0 });
                }
            }
        }
    }
    for c in cities() {
        let (cx, cy) = ll_to_tile(c.lon, c.lat);
        ensure_mine(&mut map, cx as i32, cy as i32, &mut rng);
    }
    connect_regions(&mut map);
    map
}

fn ensure_forest(map: &mut Map, cx: i32, cy: i32, rng: &mut Rng) {
    let mut cnt = 0;
    for dy in -10..=10 {
        for dx in -10..=10 {
            if map.t(cx + dx, cy + dy) == Terrain::Forest {
                cnt += 1;
            }
        }
    }
    if cnt >= 14 {
        return;
    }
    for _ in 0..30 {
        let ang = rng.range(0.0, 6.283);
        let d = rng.range(5.0, 8.0);
        let fx = cx + (ang.cos() * d) as i32;
        let fy = cy + (ang.sin() * d) as i32;
        let mut placed = 0;
        for dy in -2..=2 {
            for dx in -2..=2 {
                if dx * dx + dy * dy > 5 {
                    continue;
                }
                let (x, y) = (fx + dx, fy + dy);
                if Map::inb(x, y) && matches!(map.terrain[Map::idx(x, y)], Terrain::Plain | Terrain::Steppe | Terrain::Desert) {
                    map.terrain[Map::idx(x, y)] = Terrain::Forest;
                    placed += 1;
                }
            }
        }
        if placed > 8 {
            return;
        }
    }
}

fn ensure_mine(map: &mut Map, cx: i32, cy: i32, rng: &mut Rng) {
    if map.mines.iter().any(|m| (m.x - cx).abs().max((m.y - cy).abs()) <= 12) {
        return;
    }
    for _ in 0..40 {
        let ang = rng.range(0.0, 6.283);
        let d = rng.range(6.0, 9.0);
        let x = cx + (ang.cos() * d) as i32;
        let y = cy + (ang.sin() * d) as i32;
        if Map::inb(x, y) && map.terrain[Map::idx(x, y)].passable() && map.terrain[Map::idx(x, y)] != Terrain::River {
            map.terrain[Map::idx(x, y)] = Terrain::Hill;
            map.mines.push(Mine { x, y, amount: 2500.0 });
            return;
        }
    }
}

/// 通行可能なマスの連結成分（4近傍）。番号と大きさを返す
pub fn components(pass: impl Fn(usize) -> bool) -> (Vec<u32>, Vec<usize>) {
    let n = MAP_W * MAP_H;
    let mut comp = vec![u32::MAX; n];
    let mut sizes: Vec<usize> = Vec::new();
    let mut stack = Vec::new();
    for s in 0..n {
        if comp[s] != u32::MAX || !pass(s) {
            continue;
        }
        let id = sizes.len() as u32;
        stack.push(s);
        comp[s] = id;
        let mut sz = 0;
        while let Some(c) = stack.pop() {
            sz += 1;
            let (x, y) = ((c % MAP_W) as i32, (c / MAP_W) as i32);
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (nx, ny) = (x + dx, y + dy);
                if Map::inb(nx, ny) {
                    let ni = Map::idx(nx, ny);
                    if comp[ni] == u32::MAX && pass(ni) {
                        comp[ni] = id;
                        stack.push(ni);
                    }
                }
            }
        }
        sizes.push(sz);
    }
    (comp, sizes)
}

/// 山に囲まれて孤立した盆地に峠を開ける（海に面した島はそのまま残す：渡るには船が要る）
fn connect_regions(map: &mut Map) {
    for _ in 0..64 {
        let n = MAP_W * MAP_H;
        let (comp, sizes) = components(|i| map.terrain[i].passable());
        if sizes.len() <= 1 {
            return;
        }
        let main = sizes.iter().enumerate().max_by_key(|(_, s)| **s).unwrap().0 as u32;
        // 海に面しているか
        let mut coastal = vec![false; sizes.len()];
        for i in 0..n {
            let c = comp[i];
            if c == u32::MAX || coastal[c as usize] {
                continue;
            }
            let (x, y) = ((i % MAP_W) as i32, (i / MAP_W) as i32);
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                if Map::inb(x + dx, y + dy) && map.terrain[Map::idx(x + dx, y + dy)] == Terrain::Sea {
                    coastal[c as usize] = true;
                }
            }
        }
        let mut target = None;
        for (id, &sz) in sizes.iter().enumerate() {
            if id as u32 != main && sz >= 12 && !coastal[id] {
                target = Some(id as u32);
                break;
            }
        }
        let Some(tid) = target else {
            // 小さな孤立領域（島ではないもの）は山にする
            for i in 0..n {
                let c = comp[i];
                if c != u32::MAX && c != main && !coastal[c as usize] {
                    map.terrain[i] = Terrain::Mountain;
                }
            }
            return;
        };
        let start = (0..n).find(|&i| comp[i] == tid).unwrap();
        let (sx, sy) = ((start % MAP_W) as i32, (start / MAP_W) as i32);
        let mut best = (i32::MAX, 0usize);
        for i in 0..n {
            if comp[i] == main {
                let (x, y) = ((i % MAP_W) as i32, (i / MAP_W) as i32);
                let d = (x - sx).pow(2) + (y - sy).pow(2);
                if d < best.0 {
                    best = (d, i);
                }
            }
        }
        let (ex, ey) = ((best.1 % MAP_W) as i32, (best.1 / MAP_W) as i32);
        let steps = ((ex - sx).abs().max((ey - sy).abs())).max(1);
        for s in 0..=steps {
            let x = sx + (ex - sx) * s / steps;
            let y = sy + (ey - sy) * s / steps;
            for (dx, dy) in [(0, 0), (1, 0), (0, 1)] {
                if Map::inb(x + dx, y + dy) {
                    let i = Map::idx(x + dx, y + dy);
                    if !map.terrain[i].passable() {
                        map.terrain[i] = if map.terrain[i] == Terrain::Mountain { Terrain::Hill } else { Terrain::River };
                    }
                }
            }
        }
    }
}
