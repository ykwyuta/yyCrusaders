//! 史実イベント：遠征ごとの年表。英雄・名将・政治指導者の登場、反乱・簒奪による君主家の交代、
//! 史実の都市の奪取、王朝の交代、英雄の帰国・暗殺
//!
//! 年表は西暦（月を小数で表す）で並べ、その時が来たら発生する。いまの遠征の出来事だけが起こる。
//! 対象国が滅亡（または登場しない）していれば省略する。

use super::*;

/// 遠征の始まりの君主家
pub fn initial_house(s: Scen, n: usize) -> &'static str {
    match (n, s) {
        (CRUSADERS, Scen::First) => "諸侯の連合",
        (CRUSADERS, Scen::Second) => "アンジュー家（エルサレム王家）",
        (CRUSADERS, _) => "諸王の連合",
        (FATIMID, _) => "ファーティマ家",
        (AYYUBID, _) => "アイユーブ家",
        (BYZANTIUM, _) => "コムネノス家",
        (RUM, _) => "ルーム・セルジューク家",
        (GREAT_SELJUK, _) => "セルジューク家",
        (DANISHMEND, _) => "ダニシュメンド家",
        (DAMASCUS, Scen::First) => "セルジューク家（ドゥカーク）",
        (DAMASCUS, _) => "ブーリー家",
        (ALEPPO, _) => "セルジューク家（リドワーン）",
        (ARTUQID, _) => "アルトゥク家",
        (ZENGID, _) => "ザンギー家",
        (ABBASID, _) => "アッバース家",
        (ARMENIA, _) => "ルベン家",
        (VENICE, _) => "ドージェ（総督）",
        (GENOA, _) => "コンパーニャ（自治団）",
        (PISA, _) => "コンソリ（執政）",
        (FRANCE, _) => "カペー家",
        (HRE, _) => "ザーリアー家",
        (HUNGARY, _) => "アールパード家",
        (NORMANS, _) => "オートヴィル家",
        (ENGLAND, _) => "ノルマン家",
        (CASTILE, _) => "ヒメノ家",
        (ALMORAVID, _) => "ムラービト家",
        (ZIRID, _) => "ズィール家",
        _ => "",
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(dead_code)]
pub enum HKind {
    /// 英雄・名将・名君
    Hero,
    /// 学者・宗教者
    Sage,
    /// 政治指導者（宰相・改革者）
    Minister,
    /// 反乱・内乱
    Revolt,
    /// 簒奪・乗っ取り（君主家の交代）
    Usurp,
    /// 旧君主家の復辟
    Restore,
    /// 大敗・災厄
    Disaster,
}

impl HKind {
    pub fn label(self) -> &'static str {
        match self {
            HKind::Hero => t!("英雄", "Hero", "英雄"),
            HKind::Sage => t!("賢者", "Sage", "贤者"),
            HKind::Minister => t!("政治指導者", "Statesman", "政治家"),
            HKind::Revolt => t!("反乱", "Revolt", "叛乱"),
            HKind::Usurp => t!("簒奪", "Usurpation", "篡位"),
            HKind::Restore => t!("復辟", "Restoration", "复辟"),
            HKind::Disaster => t!("大敗", "Disaster", "大败"),
        }
    }
    /// 国にとって不利な出来事か
    pub fn bad(self) -> bool {
        matches!(self, HKind::Revolt | HKind::Usurp | HKind::Disaster)
    }
    pub fn color(self) -> [f32; 3] {
        if self.bad() {
            [1.0, 0.55, 0.45]
        } else {
            [0.75, 0.9, 1.0]
        }
    }
}

/// イベントの効果（対象は原則としてイベントの国）
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum Fx {
    /// 人物が仕える：生産（税・採集）と兵の攻撃力を一定年数だけ高める（負なら混乱）
    Figure(&'static str, f32, f32, f32),
    /// 人物が去る
    Dismiss(&'static str),
    /// 資源を得る
    Grant(f32, f32, f32),
    /// 別の国が資源を得る
    GrantTo(usize, f32, f32, f32),
    /// 全兵の能力を上げる（時代上限まで）
    Train(f32),
    /// 都に兵を召集
    Levy(u32),
    /// 兵の一部が離反・戦死する
    Purge(f32),
    /// 蓄えの一部を失う
    Plunder(f32),
    /// 君主家（国姓）が変わる
    House(&'static str),
    /// 指定国を標的とした反〇〇同盟（隣国が結ぶ）
    Hezong(usize),
    /// 自国を標的とする連合を解散させる（連衡）
    Breakup,
    /// a と b が休戦する
    Truce(usize, usize, f32),
    /// a が b に宣戦する
    War(usize, usize),
    /// 生産能力バイアスを掛ける（国の盛衰）
    Bias(f32),
    /// 指定国を服属させる（城邑と兵を併せる）
    Annex(usize),
    /// 名前の都市を、その国（第2項）から奪う
    Seize(&'static str, usize),
    /// 国の名と色が NATIONS の別の国のものになる（王朝の交代）
    Become(usize),
    /// 英雄が戦場を去る（名前, 理由）
    Leave(&'static str, &'static str),
}

pub struct HEvent {
    pub scen: Scen,
    /// 西暦（月を小数で表す。1096.6＝1096年8月）
    pub ad: f32,
    pub nation: usize,
    pub kind: HKind,
    pub title: &'static str,
    pub text: &'static str,
    pub fx: &'static [Fx],
}

macro_rules! ev {
    ($s:ident, $ad:expr, $n:expr, $k:ident, $t:expr, $x:expr, [$($fx:expr),* $(,)?]) => {
        HEvent { scen: Scen::$s, ad: $ad, nation: $n, kind: HKind::$k, title: $t, text: $x, fx: &[$($fx),*] }
    };
}

use Fx::*;

/// 史実年表（遠征ごとに西暦の昇順）
pub const HISTORY: &[HEvent] = &[
    // ---------------------------------------------------------------- 第1回十字軍
    ev!(First, 1096.62, CRUSADERS, Hero, "神がそれを望みたもう", "教皇ウルバヌス2世の呼びかけに応え、諸侯は十字を縫いつけた衣をまとって東へ発った。", [Figure("教皇ウルバヌス2世", 0.0, 0.15, 3.0)]),
    ev!(First, 1096.75, RUM, Hero, "キヴェトットの待ち伏せ", "クルチ・アルスラーン1世は、ボスポラスを渡った民衆十字軍を待ち伏せた。", [Train(0.5), Levy(3)]),
    ev!(First, 1096.95, ALEPPO, Revolt, "兄弟の争い", "アレッポのリドワーンとダマスカスのドゥカークの兄弟は争い、互いに助けなかった。", [Purge(0.1)]),
    ev!(First, 1097.05, BYZANTIUM, Minister, "アレクシオスへの誓約", "諸侯は皇帝アレクシオスに臣従を誓い、旧ローマ領を返すと約束した。皇帝は贈り物と食糧を与えた。", [Truce(CRUSADERS, BYZANTIUM, 8.0), GrantTo(CRUSADERS, 300.0, 100.0, 100.0)]),
    ev!(First, 1097.5, GREAT_SELJUK, Revolt, "スルタンたちの内紛", "スルタン・バルキヤールクと兄弟たちが争い、セルジューク家の諸侯は団結しなかった。", [Purge(0.15), Bias(0.9)]),
    ev!(First, 1097.7, ARMENIA, Hero, "エデッサのトロス", "エデッサのアルメニア人の君主トロスは、十字軍に助けを求めた。", [Truce(ARMENIA, CRUSADERS, 8.0), Figure("トロス", 0.1, 0.1, 3.0)]),
    ev!(First, 1098.3, GREAT_SELJUK, Hero, "ケルボガの大軍", "モスルのケルボガが諸侯の軍を集め、アンティオキアの救援に向かった。", [Levy(8), Figure("ケルボガの連合軍", 0.0, 0.2, 1.0)]),
    ev!(First, 1098.45, CRUSADERS, Hero, "聖槍の発見", "飢えた十字軍の陣で、ペトルス・バルトロメオが聖槍を見つけた。十字軍は奮い立った。", [Figure("聖槍", 0.0, 0.3, 1.0), Train(0.5)]),
    ev!(First, 1098.6, FATIMID, Hero, "エルサレムの奪回", "宰相アル＝アフダルは四十日の包囲の末、アルトゥク家からエルサレムを奪った。", [Seize("エルサレム", ARTUQID), Figure("アル＝アフダル", 0.1, 0.2, 6.0)]),
    ev!(First, 1098.62, CRUSADERS, Disaster, "アンティオキアの疫病", "夏の疫病が十字軍の陣を襲った。教皇の名代アデマール司教も世を去った。", [Leave("アデマール・ド・モンテイユ", "は疫病で世を去った"), Purge(0.06)]),
    ev!(First, 1099.0, CRUSADERS, Hero, "エルサレムへの行進", "諸侯の争いに怒った兵と民に押され、レーモンは裸足で南へ発った。", [Train(0.3)]),
    ev!(First, 1099.3, FATIMID, Minister, "籠城の備え", "エルサレムの守将イフティハールは周りの井戸に毒を入れ、キリスト教徒を城外へ追った。", [Figure("籠城の備え", 0.0, 0.15, 1.0)]),
    ev!(First, 1100.5, CRUSADERS, Disaster, "ゴドフロワの死", "聖墳墓の守護者ゴドフロワが病に倒れた。弟ボードゥアンがエデッサから来て王位に就く。", [Leave("ゴドフロワ・ド・ブイヨン", "は病に倒れた")]),
    ev!(First, 1101.7, FATIMID, Hero, "ラムラの戦い", "アル＝アフダルはエジプトから大軍を送り、十字軍の王国を脅かした。", [Levy(6)]),
    ev!(First, 1102.4, FATIMID, Hero, "第二次ラムラの戦い", "ファーティマ朝の軍がふたたび北上し、ラムラで十字軍を破った。", [Levy(6), Train(0.5)]),
    ev!(First, 1103.5, ARTUQID, Hero, "アルトゥク家の再起", "ソクマンとイルガーズィーはジャズィーラで力を蓄えた。", [Levy(4)]),
    // ---------------------------------------------------------------- 第2回十字軍
    ev!(Second, 1148.48, CRUSADERS, Minister, "アッコンの会議", "諸侯はアッコンに会し、エデッサではなくダマスカスを攻めると決めた。", [Figure("二人の王の軍", 0.0, 0.2, 1.5)]),
    ev!(Second, 1148.55, DAMASCUS, Minister, "ヌールッディーンへの援軍の要請", "ウヌルはアレッポのヌールッディーンに援軍を求めた。", [Truce(DAMASCUS, ZENGID, 10.0), War(ZENGID, CRUSADERS), Levy(6)]),
    ev!(Second, 1149.1, CRUSADERS, Disaster, "王たちの帰国", "ダマスカスを落とせなかったルイ7世とコンラート3世は、それぞれの国へ帰った。", [Leave("ルイ7世", "はフランスへ帰った"), Leave("コンラート3世", "はドイツへ帰った"), Purge(0.15)]),
    ev!(Second, 1149.5, ZENGID, Hero, "イナブの戦い", "ヌールッディーンがアンティオキア公レーモンを討ち、その首をカリフに送った。", [Figure("イナブの勝利", 0.0, 0.2, 3.0), Train(0.5)]),
    ev!(Second, 1153.6, CRUSADERS, Hero, "アスカロンの陥落", "ボードゥアン3世が七か月の包囲の末、ファーティマ朝の最後の砦アスカロンを奪った。", [Seize("アスカロン", FATIMID)]),
    ev!(Second, 1154.3, ZENGID, Hero, "ダマスカスの開城", "ダマスカスの民は門を開き、ヌールッディーンを迎え入れた。シリアは一つになった。", [Annex(DAMASCUS)]),
    ev!(Second, 1158.8, BYZANTIUM, Hero, "マヌエルのキリキア遠征", "マヌエル1世がキリキアに攻め入り、アンティオキアの君主を跪かせた。", [War(BYZANTIUM, ARMENIA), Levy(5)]),
    ev!(Second, 1164.6, ZENGID, Hero, "ハーリムの戦い", "ヌールッディーンが十字軍の諸侯を破り、アンティオキア公らを捕らえた。", [Train(0.5), Levy(4)]),
    ev!(Second, 1169.2, FATIMID, Usurp, "サラディン、宰相となる", "シールクーフの死後、甥のサラディンがファーティマ朝の宰相となった。", [House("アイユーブ家（宰相）"), Figure("サラディン", 0.3, 0.3, 24.0), Bias(1.15)]),
    ev!(Second, 1171.7, FATIMID, Usurp, "ファーティマ朝の終焉", "サラディンはカリフ・アーディドの死とともにファーティマ朝を廃し、アイユーブ朝を開いた。", [Become(AYYUBID), House("アイユーブ家"), Bias(1.15)]),
    ev!(Second, 1174.35, ZENGID, Disaster, "ヌールッディーンの死", "ヌールッディーンが世を去り、幼い子が残された。ザンギー家の諸侯は割れた。", [Purge(0.2), Bias(0.85)]),
    ev!(Second, 1174.85, FATIMID, Hero, "サラディンのダマスカス入城", "サラディンはダマスカスに入り、ヌールッディーンの遺領をまとめ始めた。", [Seize("ダマスカス", ZENGID), Seize("ホムス", ZENGID)]),
    ev!(Second, 1176.7, RUM, Hero, "ミュリオケファロンの戦い", "クルチ・アルスラーン2世が山峡でビザンツ皇帝マヌエル1世を破った。", [Figure("クルチ・アルスラーン2世", 0.1, 0.3, 16.0), Train(0.5)]),
    ev!(Second, 1176.72, BYZANTIUM, Disaster, "ミュリオケファロンの敗戦", "マヌエル1世はこの敗戦を「マンツィケルトの再来」と嘆いた。", [Purge(0.3)]),
    ev!(Second, 1180.2, ABBASID, Hero, "カリフ・ナースィル", "ナースィルが即位し、カリフの権威を立て直した。", [Figure("ナースィル", 0.3, 0.1, 45.0)]),
    ev!(Second, 1180.7, BYZANTIUM, Disaster, "マヌエル1世の死", "マヌエル1世が世を去り、コムネノス朝の栄光は終わった。", [Bias(0.85)]),
    ev!(Second, 1183.45, FATIMID, Hero, "アレッポの併合", "サラディンはアレッポを手に入れ、シリアとエジプトを一つにした。", [Seize("アレッポ", ZENGID)]),
    ev!(Second, 1185.7, BYZANTIUM, Usurp, "アンゲロス家の帝位", "暴君アンドロニコス1世が倒され、イサキオス2世アンゲロスが帝位に就いた。", [House("アンゲロス家"), Purge(0.15)]),
    ev!(Second, 1187.5, FATIMID, Hero, "ヒッティーンの戦い", "サラディンがヒッティーンで十字軍を破り、聖十字架を奪った。", [War(FATIMID, CRUSADERS), Levy(6), Train(0.5)]),
    ev!(Second, 1187.52, CRUSADERS, Disaster, "ヒッティーンの敗戦", "エルサレム王国の騎士たちは壊滅し、王ギーも捕らわれた。", [Purge(0.4), Plunder(0.4)]),
    ev!(Second, 1187.75, FATIMID, Hero, "エルサレムの開城", "サラディンは無血でエルサレムに入った。住民は身代金を払って去ることを許された。", [Seize("エルサレム", CRUSADERS)]),
    ev!(Second, 1189.7, CRUSADERS, Hero, "第3回十字軍", "リチャード獅子心王とフィリップ2世がアッコンの包囲の陣に着いた。", [Figure("リチャード獅子心王", 0.0, 0.4, 3.0), Levy(10), Grant(400.0, 300.0, 300.0)]),
    ev!(Second, 1191.4, CRUSADERS, Hero, "キプロスの征服", "リチャードはキプロスの僭主イサキオスを破り、島を手に入れた。", [Seize("ニコシア", BYZANTIUM)]),
    ev!(Second, 1191.55, CRUSADERS, Hero, "アッコンの奪回", "二年にわたる包囲の末、アッコンは十字軍に降った。", [Seize("アッコン", FATIMID)]),
    ev!(Second, 1192.67, CRUSADERS, Minister, "ヤッファの和約", "リチャードとサラディンは三年の休戦を結んだ。巡礼者はエルサレムを訪れることを許された。", [Truce(CRUSADERS, FATIMID, 5.0)]),
    ev!(Second, 1193.2, FATIMID, Disaster, "サラディンの死", "サラディンがダマスカスで世を去り、アイユーブ家の諸侯は領土を分けて争った。", [Dismiss("サラディン"), Purge(0.15), Bias(0.9)]),
    ev!(Second, 1198.0, ARMENIA, Hero, "キリキア・アルメニア王国", "レヴォン1世が王冠を受け、キリキアは王国となった。", [Figure("レヴォン1世", 0.2, 0.2, 21.0), Bias(1.1)]),
    ev!(Second, 1204.3, BYZANTIUM, Disaster, "第4回十字軍", "十字軍がコンスタンティノープルを略奪し、ラテン帝国を建てた。皇帝はニカイアへ逃れた。", [House("ラスカリス家（ニカイア）"), Purge(0.5), Plunder(0.6), Bias(0.7)]),
    ev!(Second, 1207.2, RUM, Hero, "アッタレイアの占領", "カイホスロー1世が地中海の港アッタレイアを奪った。", [Seize("アッタレイア", BYZANTIUM)]),
    ev!(Second, 1218.9, FATIMID, Hero, "スルタン・カーミル", "アル＝カーミルがスルタンとなり、ダミエッタを囲む第5回十字軍に立ち向かった。", [Figure("アル＝カーミル", 0.2, 0.2, 20.0)]),
    ev!(Second, 1219.85, CRUSADERS, Hero, "ダミエッタの占領", "第5回十字軍がナイルの河口の町ダミエッタを奪った。", [Seize("ダミエッタ", FATIMID)]),
    ev!(Second, 1221.7, FATIMID, Hero, "マンスーラの勝利", "ナイルの洪水に阻まれた十字軍は降伏し、ダミエッタを返した。", [Seize("ダミエッタ", CRUSADERS)]),
    ev!(Second, 1229.15, CRUSADERS, Minister, "フリードリヒ2世の交渉", "皇帝フリードリヒ2世が、スルタン・カーミルとの交渉で一兵も損なわずにエルサレムを取り戻した。", [Seize("エルサレム", FATIMID), Truce(CRUSADERS, FATIMID, 10.0)]),
    ev!(Second, 1243.5, RUM, Disaster, "キョセ・ダグの戦い", "ルーム・セルジューク朝はモンゴルに大敗し、その属国となった。", [Purge(0.5), Bias(0.75)]),
    ev!(Second, 1244.6, CRUSADERS, Disaster, "エルサレムの喪失", "ホラズムの残党の騎兵がエルサレムを略奪し、ラ・フォルビーで十字軍は壊滅した。", [Seize("エルサレム", CRUSADERS), Purge(0.4)]),
    ev!(Second, 1249.45, CRUSADERS, Hero, "第7回十字軍", "フランス王ルイ9世がダミエッタに上陸した。", [Seize("ダミエッタ", FATIMID), Levy(6)]),
    // ---------------------------------------------------------------- 第3回十字軍
    ev!(Third, 1191.44, CRUSADERS, Hero, "リチャードの上陸", "リチャード獅子心王が二十五隻の船とともにアッコンの陣に着いた。陣は歓呼に包まれた。", [Figure("リチャードの到着", 0.0, 0.2, 0.5), Grant(300.0, 200.0, 200.0)]),
    ev!(Third, 1191.5, AYYUBID, Disaster, "アッコンの飢え", "海も陸も塞がれたアッコンの守備隊は、飢えと疲れに苦しんだ。", [Plunder(0.2)]),
    ev!(Third, 1191.6, CRUSADERS, Minister, "フィリップ2世の帰国", "フィリップ2世は病を理由にフランスへ帰り、兵の多くをブルゴーニュ公に託した。", [Leave("フィリップ2世", "はフランスへ帰った")]),
    ev!(Third, 1191.68, AYYUBID, Minister, "アル＝アーディルの交渉", "サラディンの弟アル＝アーディルはリチャードと和平を話し合った。", [Figure("和平の交渉", 0.1, 0.0, 0.5)]),
    ev!(Third, 1191.75, CRUSADERS, Hero, "南への行軍", "リチャードは海沿いに兵を進めた。歩兵の盾の列が騎士を守り、艦隊が並んで進む。", [Figure("行軍の隊形", 0.0, 0.15, 0.4)]),
    ev!(Third, 1191.8, AYYUBID, Hero, "タキー・アッディーンの死", "ハマの領主タキー・アッディーンが世を去った。", [Leave("タキー・アッディーン", "は世を去った")]),
    ev!(Third, 1192.0, CRUSADERS, Disaster, "冬の雨", "冬の雨と寒さが十字軍を苦しめた。エルサレムを前にして十字軍は引き返した。", [Purge(0.05)]),
    ev!(Third, 1192.32, CRUSADERS, Disaster, "コンラートの暗殺", "エルサレム王に選ばれたばかりのコンラートが、ニザール派の刺客に刺された。", [Leave("コンラート・ド・モンフェラート", "はニザール派の刺客に暗殺された")]),
    ev!(Third, 1192.5, AYYUBID, Hero, "ヤッファの急襲", "サラディンはヤッファを急襲した。", [Levy(5)]),
    ev!(Third, 1192.55, CRUSADERS, Hero, "リチャードのヤッファ救援", "リチャードは海から駆けつけ、わずかな兵でヤッファを救った。", [Train(0.5)]),
];

/// 在世の人物（効果は年限つき）
pub struct Person {
    pub name: &'static str,
    pub eco: f32,
    pub mil: f32,
    pub until: f32,
}

/// 史書の記録
pub struct Chron {
    pub ad: i32,
    pub nation: usize,
    pub kind: HKind,
    pub title: &'static str,
    pub text: &'static str,
    /// 対象国が滅んでいて起こらなかった
    pub skipped: bool,
}

impl World {
    /// 年が改まるごとに史実イベントを発生させる
    pub(super) fn run_history(&mut self) {
        let ad = self.ad();
        // 人物の退場
        let years = self.years;
        for n in self.nations.iter_mut() {
            n.figures.retain(|f| f.until > years);
        }
        let s = self.scen;
        while self.hist_i < HISTORY.len() && (HISTORY[self.hist_i].scen != s || HISTORY[self.hist_i].ad <= ad) {
            let e = &HISTORY[self.hist_i];
            self.hist_i += 1;
            if e.scen == s {
                self.fire_event(e);
            }
        }
        self.refresh_mods();
    }

    fn fire_event(&mut self, e: &'static HEvent) {
        let n = e.nation;
        let skipped = !self.nations[n].alive;
        self.chronicle.push(Chron { ad: e.ad.floor() as i32, nation: n, kind: e.kind, title: e.title, text: e.text, skipped });
        if skipped {
            return;
        }
        // チュートリアル中はプレイヤーに不利な出来事を起こさない
        let shield = self.protect_player && n == self.player;
        let mut notes: Vec<String> = Vec::new();
        for fx in e.fx.iter() {
            match *fx {
                Figure(name, eco, mil, yrs) => {
                    if shield && (eco < 0.0 || mil < 0.0) {
                        continue;
                    }
                    let until = self.years + yrs;
                    let nat = &mut self.nations[n];
                    nat.figures.retain(|f| f.name != name);
                    nat.figures.push(Person { name, eco, mil, until });
                    let mut s = String::from(tx(name));
                    if eco != 0.0 {
                        s.push_str(&tf!(" 生産{:+.0}%", " production {:+.0}%", " 生产{:+.0}%", eco * 100.0));
                    }
                    if mil != 0.0 {
                        s.push_str(&tf!(" 武{:+.0}%", " military {:+.0}%", " 武{:+.0}%", mil * 100.0));
                    }
                    if yrs >= 1.0 {
                        s.push_str(&tf!("（{}年）", " ({} years)", "（{}年）", yrs.min(self.end_ad() - self.start_ad()) as i32));
                    } else {
                        s.push_str(&tf!("（{}か月）", " ({} months)", "（{}个月）", (yrs * 12.0).round() as i32));
                    }
                    notes.push(s);
                }
                Dismiss(name) => {
                    let nat = &mut self.nations[n];
                    if nat.figures.iter().any(|f| f.name == name) {
                        nat.figures.retain(|f| f.name != name);
                        notes.push(tf!("{} が去る", "{} departs", "{}离去", tx(name)));
                    }
                }
                Grant(a, b, c) => {
                    let r = &mut self.nations[n].res;
                    r[0] += a;
                    r[1] += b;
                    r[2] += c;
                    notes.push(tf!("糧{:.0} 木{:.0} 鉄{:.0}", "food {:.0}, wood {:.0}, iron {:.0}", "粮{:.0} 木{:.0} 铁{:.0}", a, b, c));
                }
                Train(x) => {
                    let cap = self.power_cap();
                    let mut k = 0;
                    for u in self.units.iter_mut() {
                        if u.alive && u.nation as usize == n && u.kind.military() && u.power < cap {
                            let ratio = u.hp / u.max_hp();
                            u.power = (u.power + x).min(cap);
                            u.hp = ratio * u.max_hp();
                            k += 1;
                        }
                    }
                    if k > 0 {
                        notes.push(tf!("兵{}人の能力{:+.1}", "{} soldiers' power {:+.1}", "{}名士兵能力{:+.1}", k, x));
                    }
                }
                Levy(cnt) => {
                    let Some(c) = self.nations[n].capital else { continue };
                    let pw = (1.0 + self.years / (self.end_ad() - self.start_ad()) * 2.0).min(self.power_cap());
                    let ut = best_land_ut(n);
                    let mut k = 0;
                    for _ in 0..cnt {
                        if let Some(sp) = self.spawn_point(c) {
                            self.spawn_unit(n, ut, sp, pw);
                            k += 1;
                        }
                    }
                    if k > 0 {
                        notes.push(tf!("{}{}人が馳せ参じる", "{1} {0} rally to the capital", "{}{}名赶来效命", tx(udef(ut).name), k));
                    }
                }
                Bias(f) => {
                    if shield && f < 1.0 {
                        continue;
                    }
                    let nat = &mut self.nations[n];
                    let old = nat.bias;
                    nat.bias = (nat.bias * f).clamp(0.4, 2.4);
                    notes.push(tf!("国力 ×{:.2} → ×{:.2}", "national strength ×{:.2} → ×{:.2}", "国力 ×{:.2} → ×{:.2}", old, nat.bias));
                }
                Annex(t) => {
                    if let Some(s) = self.annex(n, t) {
                        notes.push(s);
                    }
                }
                Seize(city, from) => {
                    if let Some(s) = self.seize_city(n, city, from) {
                        notes.push(s);
                    }
                }
                Become(o) => {
                    if let Some(s) = self.become_nation(n, o) {
                        notes.push(s);
                    }
                }
                Leave(name, why) => {
                    if let Some(s) = self.hero_leave(name, why) {
                        notes.push(s);
                    }
                }
                GrantTo(o, a, b, c) => {
                    if !self.nations[o].alive {
                        continue;
                    }
                    let r = &mut self.nations[o].res;
                    r[0] += a;
                    r[1] += b;
                    r[2] += c;
                    notes.push(tf!("{}が糧{:.0} 木{:.0} 鉄{:.0}を得る", "{} receives food {:.0}, wood {:.0}, iron {:.0}", "{}获得粮{:.0} 木{:.0} 铁{:.0}", tx(self.nations[o].name), a, b, c));
                }
                Purge(f) => {
                    if shield {
                        continue;
                    }
                    let mut idx: Vec<usize> = (0..self.units.len()).filter(|&i| self.units[i].alive && self.units[i].nation as usize == n && self.units[i].kind.military()).collect();
                    // 都から遠い兵ほど離反しやすい
                    let home = self.nations[n].home;
                    idx.sort_by(|&a, &b| self.units[b].pos.dist(home).partial_cmp(&self.units[a].pos.dist(home)).unwrap());
                    let k = (idx.len() as f32 * f).round() as usize;
                    for &i in idx.iter().take(k) {
                        self.kill_unit(i);
                    }
                    if k > 0 {
                        notes.push(tf!("兵{}人を失う", "{} soldiers lost", "损失{}名士兵", k));
                    }
                }
                Plunder(f) => {
                    if shield {
                        continue;
                    }
                    for r in self.nations[n].res.iter_mut() {
                        *r *= 1.0 - f;
                    }
                    notes.push(tf!("蓄えの{:.0}%を失う", "{:.0}% of stores lost", "损失{:.0}%的储备", f * 100.0));
                }
                House(h) => {
                    if self.nations[n].house != h {
                        notes.push(tf!("君主家：{} → {}", "ruling house: {} → {}", "君主家：{} → {}", tx(self.nations[n].house), tx(h)));
                        self.nations[n].house = h;
                    }
                }
                Hezong(t) => {
                    if self.nations[t].alive && !self.coalitions.iter().any(|c| c.target == t) && self.start_coalition_ex(CoalKind::Hezong, t, false) {
                        let c = self.coalitions.last().unwrap();
                        notes.push(tf!("隣国{}か国が反{}同盟を結ぶ", "{} neighbours form the Anti-{} League", "{}个邻国结成反{}同盟", c.members.len(), tx(self.nations[t].name)));
                    }
                }
                Breakup => {
                    let years = self.years;
                    let mut any = false;
                    for c in self.coalitions.iter_mut() {
                        if c.target == n && c.end > years {
                            c.end = years;
                            any = true;
                        }
                    }
                    if any {
                        notes.push(tf!("{}を討つ連合が瓦解", "the coalition against {} collapses", "讨伐{}的联军瓦解", tx(self.nations[n].name)));
                    }
                }
                Truce(a, b, yrs) => {
                    if !self.nations[a].alive || !self.nations[b].alive || a == b {
                        continue;
                    }
                    if self.coalition_locked(a, b) {
                        continue;
                    }
                    // 交戦中のプレイヤーには休戦を受けるか選ばせる
                    if self.war[a][b] && !self.propose_peace(a, b, yrs, tx(e.title)) {
                        notes.push(tf!("{}と{}に{}年の休戦の勧め", "{} and {} are urged to a {}-year truce", "劝{}与{}休战{}年", tx(self.nations[a].name), tx(self.nations[b].name), yrs as i32));
                        continue;
                    }
                    let t = self.years + yrs;
                    self.truce[a][b] = self.truce[a][b].max(t);
                    self.truce[b][a] = self.truce[b][a].max(t);
                    notes.push(tf!("{}と{}が{}年の休戦", "{} and {} agree a {}-year truce", "{}与{}休战{}年", tx(self.nations[a].name), tx(self.nations[b].name), yrs as i32));
                }
                War(a, b) => {
                    // プレイヤーの国に勝手に宣戦させない
                    if a == self.player || (self.protect_player && b == self.player) {
                        continue;
                    }
                    if self.declare_war(a, b, true) {
                        notes.push(tf!("{}が{}に攻め込む", "{} invades {}", "{}进攻{}", tx(self.nations[a].name), tx(self.nations[b].name)));
                    }
                }
            }
        }
        self.refresh_mods();
        self.territory_dirty = true;
        let name = tx(self.nations[n].name);
        let head = tf!("【{}・{}】{}", "[{} · {}] {}", "【{}・{}】{}", name, e.kind.label(), tx(e.title));
        // 本文はバナーと史記に任せ、記録欄は見出しと効果だけにする
        let mut msg = head;
        if !notes.is_empty() {
            msg.push_str(&tf!("（{}）", " ({})", "（{}）", notes.join(t!("、", "; ", "；"))));
        }
        self.push_msg(msg, e.kind.color());
        // 大きな出来事はバナーで知らせる（小国の出来事は記録のみ）
        let involves_player = n == self.player
            || e.fx.iter().any(|f| match *f {
                Hezong(t) | Annex(t) | Seize(_, t) | GrantTo(t, ..) => t == self.player,
                War(a, b) | Truce(a, b, _) => a == self.player || b == self.player,
                _ => false,
            });
        let annexes_player = e.fx.iter().any(|f| matches!(*f, Annex(t) if t == self.player));
        if involves_player || annexes_player || NATIONS[n].major {
            let mut text = tx(e.text).to_string();
            if !notes.is_empty() {
                text.push_str(&format!("\n{}", notes.join(t!("　", "; ", "；"))));
            }
            let title = format!("{}　{}", name, tx(e.title));
            self.queue_banner(Banner { title, text, t: 7.0 });
        }
    }

    /// 在世の人物の効果を集計する
    pub(super) fn refresh_mods(&mut self) {
        let years = self.years;
        for n in self.nations.iter_mut() {
            let (mut e, mut m) = (1.0f32, 1.0f32);
            for f in n.figures.iter().filter(|f| f.until > years) {
                e += f.eco;
                m += f.mil;
            }
            n.eco_mul = e.clamp(0.4, 3.0);
            n.mil_mul = m.clamp(0.5, 2.5);
        }
    }

    /// いまの告知を閉じて、次の告知へ
    pub fn dismiss_banner(&mut self) {
        self.banner = self.banner_queue.pop_front();
    }

    /// 待っている告知の数（いま出ているものを除く）
    pub fn banners_waiting(&self) -> usize {
        self.banner_queue.len()
    }

    /// 告知をすべて閉じる（記録欄と史記には残る）
    pub fn dismiss_all_banners(&mut self) {
        self.banner = None;
        self.banner_queue.clear();
    }

    pub fn queue_banner(&mut self, b: Banner) {
        if self.banner.is_none() {
            self.banner = Some(b);
        } else {
            self.banner_queue.push_back(b);
        }
    }
}
