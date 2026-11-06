//! 国ごとの史実に基づく特徴

use super::*;

/// 特徴による補正（1.0＝補正なし）
#[derive(Clone, Copy, Debug)]
pub struct TraitMods {
    /// 生産（城邑の税・採集）
    pub eco: f32,
    /// 兵の大分類ごとの攻撃（民・歩兵・弓兵・騎兵・船）
    pub atk: [f32; 5],
    /// 兵の大分類ごとの訓練費
    pub cost: [f32; 5],
    /// 軍の維持費
    pub upkeep: f32,
    /// 城邑・建物が受ける損害
    pub def: f32,
    /// 商人の集まりやすさ
    pub attract: f32,
    /// 市場の値
    pub market: f32,
    /// 部族の略奪で失う蓄え
    pub plunder: f32,
    /// 鍛錬・教化の費用
    pub upgrade: f32,
}

impl TraitMods {
    pub const NONE: TraitMods = TraitMods { eco: 1.0, atk: [1.0; 5], cost: [1.0; 5], upkeep: 1.0, def: 1.0, attract: 1.0, market: 1.0, plunder: 1.0, upgrade: 1.0 };
}

pub struct Trait {
    pub nation: usize,
    pub name: &'static str,
    pub desc: &'static str,
    /// この年（西暦）から有効
    pub from_ad: i32,
    /// この年（西暦）まで有効
    pub to_ad: i32,
    pub fx: fn(&mut TraitMods),
}

const ALL: (i32, i32) = (1090, 1260);
const INF: usize = 1;
const CAVK: usize = 3;
const SHIP: usize = 4;

macro_rules! tr {
    ($n:expr, $name:expr, $desc:expr, $span:expr, $f:expr) => {
        Trait { nation: $n, name: $name, desc: $desc, from_ad: $span.0, to_ad: $span.1, fx: $f }
    };
}

fn horse(m: &mut TraitMods) {
    m.atk[CAVK] *= 1.2;
    m.cost[CAVK] *= 0.8;
}

fn sea(m: &mut TraitMods) {
    m.atk[SHIP] *= 1.25;
    m.cost[SHIP] *= 0.8;
    m.attract *= 1.4;
}

pub const TRAITS: &[Trait] = &[
    tr!(CRUSADERS, "神がそれを望みたもう", "信仰に燃える騎士と民。騎兵と歩兵の攻撃+15%", ALL, |m| {
        m.atk[INF] *= 1.15;
        m.atk[CAVK] *= 1.15;
    }),
    tr!(CRUSADERS, "城の国", "十字軍国家は石の城で国を守った。城邑・建物の受ける損害-20%", (1100, 1260), |m| m.def *= 0.8),
    tr!(FATIMID, "ナイルの富", "ナイルの実りと紅海の交易で栄えた。生産+20%、商人が集まる（×1.3）", ALL, |m| {
        m.eco *= 1.2;
        m.attract *= 1.3;
    }),
    tr!(FATIMID, "地中海の艦隊", "エジプトは東地中海で最大の艦隊を持った。船の攻撃+20%", ALL, |m| m.atk[SHIP] *= 1.2),
    tr!(AYYUBID, "聖戦の旗", "サラディンはエジプトとシリアの兵をまとめた。兵の攻撃+10%、維持費-10%", ALL, |m| {
        for k in 1..4 {
            m.atk[k] *= 1.1;
        }
        m.upkeep *= 0.9;
    }),
    tr!(AYYUBID, "ナイルの富", "エジプトの富がシリアの戦を支えた。生産+15%", ALL, |m| m.eco *= 1.15),
    tr!(BYZANTIUM, "テオドシウスの城壁", "コンスタンティノープルの三重の城壁。城邑・建物の受ける損害-25%", ALL, |m| m.def *= 0.75),
    tr!(BYZANTIUM, "ギリシアの火", "水の上でも燃える秘密の火。船の攻撃+30%", ALL, |m| m.atk[SHIP] *= 1.3),
    tr!(BYZANTIUM, "ノミスマ金貨", "千年の都の金貨は地中海の通貨だった。商人が集まる（×1.5）、市場の値+20%", ALL, |m| {
        m.attract *= 1.5;
        m.market *= 1.2;
    }),
    tr!(RUM, "騎馬の民", "アナトリアのトルコ人の遊牧騎兵。騎兵の攻撃+20%、騎兵の訓練費-20%", ALL, horse),
    tr!(GREAT_SELJUK, "騎馬の民", "トルコ人の大帝国。騎兵の攻撃+20%、騎兵の訓練費-20%", ALL, horse),
    tr!(GREAT_SELJUK, "イクター制", "将兵に土地の徴税権を与えて軍を養った。維持費-20%", ALL, |m| m.upkeep *= 0.8),
    tr!(DANISHMEND, "騎馬の民", "アナトリア中部のトルコ人。騎兵の攻撃+20%、騎兵の訓練費-20%", ALL, horse),
    tr!(DAMASCUS, "ダマスカスの城壁", "グータの果樹園と城壁に守られた古都。城邑・建物の受ける損害-20%、生産+10%", ALL, |m| {
        m.def *= 0.8;
        m.eco *= 1.1;
    }),
    tr!(ALEPPO, "城塞の町", "丘の上の城塞に拠るシリアの町。城邑・建物の受ける損害-20%", ALL, |m| m.def *= 0.8),
    tr!(ARTUQID, "トルクメンの騎兵", "ジャズィーラのトルクメン。騎兵の攻撃+20%、騎兵の訓練費-20%", ALL, horse),
    tr!(ZENGID, "聖戦", "ヌールッディーンは十字軍への聖戦を掲げた。兵の攻撃+15%、鍛錬の費用-20%", ALL, |m| {
        for k in 1..4 {
            m.atk[k] *= 1.15;
        }
        m.upgrade *= 0.8;
    }),
    tr!(ABBASID, "カリフの権威", "スンナ派の信徒の長。商人が集まる（×1.5）、鍛錬・教化の費用-30%", ALL, |m| {
        m.attract *= 1.5;
        m.upgrade *= 0.7;
    }),
    tr!(ARMENIA, "山の城", "タウルスの山城に拠ったアルメニア人。城邑・建物の受ける損害-25%", ALL, |m| m.def *= 0.75),
    tr!(VENICE, "海の共和国", "地中海の商業を握った。船の攻撃+25%、船の訓練費-20%、商人が集まる（×1.4）", ALL, sea),
    tr!(GENOA, "海の共和国", "地中海の商業を握った。船の攻撃+25%、船の訓練費-20%、商人が集まる（×1.4）", ALL, sea),
    tr!(PISA, "海の共和国", "地中海の商業を握った。船の攻撃+25%、船の訓練費-20%、商人が集まる（×1.4）", ALL, sea),
    tr!(FRANCE, "騎士の国", "重装の騎士の突撃。騎兵の攻撃+15%", ALL, |m| m.atk[CAVK] *= 1.15),
    tr!(HRE, "騎士の国", "重装の騎士の突撃。騎兵の攻撃+15%", ALL, |m| m.atk[CAVK] *= 1.15),
    tr!(NORMANS, "ノルマンの騎士", "南イタリアを征した騎士。騎兵の攻撃+20%", ALL, |m| m.atk[CAVK] *= 1.2),
    tr!(HUNGARY, "騎馬の民", "マジャル人の騎兵。騎兵の攻撃+15%、騎兵の訓練費-20%", ALL, |m| {
        m.atk[CAVK] *= 1.15;
        m.cost[CAVK] *= 0.8;
    }),
    tr!(ENGLAND, "ノルマンの騎士", "ヘイスティングズで勝った騎士の国。騎兵の攻撃+15%", ALL, |m| m.atk[CAVK] *= 1.15),
    tr!(CASTILE, "レコンキスタ", "イスラーム勢力と戦い続けた。兵の攻撃+10%", ALL, |m| {
        for k in 1..4 {
            m.atk[k] *= 1.1;
        }
    }),
    tr!(ALMORAVID, "鼓の軍勢", "鼓を打ち鳴らして盾の列が進んだ。歩兵の攻撃+20%", ALL, |m| m.atk[INF] *= 1.2),
    tr!(ZIRID, "イフリーキヤの港", "地中海の交易の港。商人が集まる（×1.3）", ALL, |m| m.attract *= 1.3),
];

/// その年に有効な特徴
pub fn traits_of(n: usize, ad: i32) -> impl Iterator<Item = &'static Trait> {
    TRAITS.iter().filter(move |t| t.nation == n && ad >= t.from_ad && ad <= t.to_ad)
}

/// 年代を問わずその国の特徴すべて
pub fn all_traits_of(n: usize) -> impl Iterator<Item = &'static Trait> {
    TRAITS.iter().filter(move |t| t.nation == n)
}

impl World {
    /// 特徴の補正を計算し直す（毎年）
    pub(super) fn refresh_traits(&mut self) {
        let ad = self.year_ad();
        for i in 0..NATIONS.len() {
            let mut m = TraitMods::NONE;
            for t in traits_of(i, ad) {
                (t.fx)(&mut m);
            }
            self.nations[i].tm = m;
        }
    }

    /// その国の兵種の訓練費
    pub fn train_cost(&self, n: usize, ut: u8) -> [f32; 3] {
        let d = udef(ut);
        let m = if n < NATIONS.len() { self.nations[n].tm.cost[d.class as usize] } else { 1.0 };
        let c = d.cost;
        [(c[0] * m).round(), (c[1] * m).round(), (c[2] * m).round()]
    }

    /// 市場の値
    pub fn market_price(&self, n: usize, k: usize) -> f32 {
        (MARKET_PRICE[k] * self.nations[n].tm.market).round()
    }
}
