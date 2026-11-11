//! シミュレーションの試験

use super::*;

#[test]
fn discarding_units_protects_other_nations_and_passengers() {
    let mut w = World::new(12345, Scen::First);
    let pos = w.nations[CRUSADERS].home;
    let worker = w.spawn_unit(CRUSADERS, UT_WORKER, pos, 1.0);
    let troop = w.spawn_unit(CRUSADERS, UT_INF, pos, 1.0);
    let ship = w.spawn_unit(CRUSADERS, UT_GALLEY, pos, 1.0);
    let foreign = w.spawn_unit(BYZANTIUM, UT_INF, pos, 1.0);
    w.unit_mut(troop).unwrap().aboard = Some(ship);
    w.unit_mut(ship).unwrap().cargo.push(troop);
    w.update_stats();
    let workers = w.nations[CRUSADERS].workers;

    assert!(!w.discard_units(CRUSADERS, &[worker, foreign]));
    assert!(!w.discard_units(CRUSADERS, &[worker, ship]));
    assert!(w.unit(worker).is_some() && w.unit(troop).is_some());
    assert!(w.discard_units(CRUSADERS, &[worker]));
    assert!(w.unit(worker).is_none());
    assert_eq!(w.nations[CRUSADERS].workers, workers - 1);
    assert!(!w.discard_units(CRUSADERS, &[worker]));
}

#[test]
fn repair_charges_wood_and_only_restores_completed_owned_buildings() {
    let mut w = World::new(12345, Scen::First);
    let fort = w.spawn_building(CRUSADERS, BKind::Fort, 1, 1, true);
    let unfinished = w.spawn_building(CRUSADERS, BKind::Fort, 3, 1, false);
    let max = BKind::Fort.max_hp();
    w.bld_mut(fort).unwrap().hp = max - 101.0;
    w.nations[CRUSADERS].res = [0.0, 10.0, 0.0];
    assert_eq!(w.repair_cost(CRUSADERS, fort), Some([0.0, 11.0, 0.0]));
    assert!(!w.repair_building(CRUSADERS, fort));
    assert_eq!(w.bld(fort).unwrap().hp, max - 101.0);
    assert_eq!(w.nations[CRUSADERS].res[1], 10.0);

    w.nations[CRUSADERS].res[1] = 15.0;
    assert!(!w.repair_building(BYZANTIUM, fort));
    assert!(!w.repair_building(CRUSADERS, unfinished));
    assert!(w.repair_building(CRUSADERS, fort));
    assert!(!w.repair_building(CRUSADERS, fort));
    assert_eq!(w.bld(fort).unwrap().hp, max);
    assert_eq!(w.nations[CRUSADERS].res[1], 4.0);
}

/// 地図を画像（PPM）に書き出す（YY_MAP_OUT=出力の接頭辞。確かめるとき用）
#[test]
fn map_preview() {
    let Ok(out) = std::env::var("YY_MAP_OUT") else { return };
    for s in SCENS {
        let w = World::new(12345, s);
        let mut img = Vec::with_capacity(MAP_W * MAP_H * 3);
        for y in 0..MAP_H as i32 {
            for x in 0..MAP_W as i32 {
                let i = Map::idx(x, y);
                let mut c = match w.map.terrain[i] {
                    Terrain::Sea => [30, 60, 120],
                    Terrain::Lake => [50, 100, 160],
                    Terrain::River => [60, 120, 200],
                    Terrain::Plain => [110, 150, 80],
                    Terrain::Steppe => [180, 175, 110],
                    Terrain::Forest => [40, 90, 40],
                    Terrain::Hill => [150, 130, 90],
                    Terrain::Mountain => [100, 90, 85],
                    Terrain::Desert => [220, 195, 130],
                };
                let o = w.owner[i];
                if o != NONE && (o as usize) < NATIONS.len() {
                    let nc = w.nations[o as usize].color;
                    for k in 0..3 {
                        c[k] = ((c[k] as f32) * 0.55 + nc[k] * 255.0 * 0.45) as u8;
                    }
                }
                img.extend_from_slice(&c);
            }
        }
        for b in w.blds.iter().filter(|b| b.alive) {
            let col: [u8; 3] = if b.kind.city() { [255, 255, 255] } else if b.kind == BKind::Village { [255, 220, 120] } else { [0, 0, 0] };
            for dy in 0..2 {
                for dx in 0..2 {
                    let i = Map::idx(b.tx + dx, b.ty + dy) * 3;
                    img[i..i + 3].copy_from_slice(&col);
                }
            }
        }
        for u in w.units.iter().filter(|u| u.alive) {
            let (x, y) = u.pos.tile();
            if Map::inb(x, y) {
                let i = Map::idx(x, y) * 3;
                img[i..i + 3].copy_from_slice(&[255, 0, 255]);
            }
        }
        let mut f = format!("P6\n{} {}\n255\n", MAP_W, MAP_H).into_bytes();
        f.extend_from_slice(&img);
        std::fs::write(format!("{}{}.ppm", out, s as u8 + 1), f).unwrap();
    }
}

fn city_pos(w: &World, name: &str) -> V2 {
    let i = w.city_bld(name).unwrap_or_else(|| panic!("{} が無い", name));
    w.blds[i].center()
}

/// 遠征ごとに、実在の都市・選べる国・英雄がそろっている。第1回の十字軍は都市を持たずに兵だけで始まる
#[test]
fn scenario_setup() {
    for s in SCENS {
        let w = World::new(4242, s);
        let d = s.def();
        for (k, c) in d.cities.iter().enumerate() {
            let b = w.blds.iter().find(|b| b.alive && b.kind.city() && b.site as usize == k);
            let b = b.unwrap_or_else(|| panic!("{:?}: {} が置かれていない", s, c.name));
            assert_eq!(b.nation as usize, c.owner, "{}", c.name);
        }
        for p in d.picks {
            assert!(w.nations[p.nation].alive, "{:?}: {} が登場しない", s, NATIONS[p.nation].name);
            assert!(w.units.iter().any(|u| u.alive && u.nation as usize == p.nation && u.kind.military()), "{} に兵がいない", NATIONS[p.nation].name);
        }
        for (k, h) in HEROES.iter().enumerate() {
            let on = w.units.iter().any(|u| u.alive && u.hero as usize == k);
            assert_eq!(on, h.scen == s, "{:?}: {}", s, h.name);
            if on {
                let u = w.units.iter().find(|u| u.alive && u.hero as usize == k).unwrap();
                assert_eq!(u.nation as usize, h.nation);
            }
        }
        for n in 0..NATIONS.len() {
            assert_eq!(w.nations[n].alive, present(s, n), "{}", NATIONS[n].name);
        }
        assert!(w.blds.iter().any(|b| b.alive && b.kind == BKind::Village), "民家が無い");
        // 交戦
        for &(a, b) in d.wars {
            assert!(w.war[a][b] && w.war[b][a]);
        }
    }
    let w = World::new(4242, Scen::First);
    assert!(w.army_nation(CRUSADERS));
    assert_eq!(w.blds.iter().filter(|b| b.alive && b.nation as usize == CRUSADERS).count(), 0);
    let n = w.units.iter().filter(|u| u.alive && u.nation as usize == CRUSADERS && u.kind.military()).count();
    assert!(n >= 60, "十字軍の兵が少ない（{}）", n);
    assert!(w.units.iter().any(|u| u.alive && u.nation as usize == CRUSADERS && u.ut == UT_PEOPLE), "民衆十字軍がいない");
}

/// 陸の道：第1回は西欧からコンスタンティノープルを経てエルサレムまで歩いて行ける。キプロスは海の向こう
#[test]
fn connectivity() {
    let w = World::new(12345, Scen::First);
    let (x, y) = ll_to_tile(6.96, 50.94);
    let cologne = v2(x, y);
    for c in ["コンスタンティノープル", "ニカイア", "アンティオキア", "エルサレム", "カイロ", "パリ", "トゥールーズ", "バーリ"] {
        assert!(w.same_land(cologne, city_pos(&w, c)), "ケルンから{}へ歩いて行けない", c);
    }
    assert!(!w.same_land(cologne, city_pos(&w, "ニコシア")), "キプロスが陸続き");
    assert!(!w.same_land(cologne, city_pos(&w, "ロンドン")), "ブリテンが陸続き");
    // 経路でも行ける（ボスポラスを渡る）
    let a = city_pos(&w, "コンスタンティノープル").tile();
    let b = city_pos(&w, "ニカイア").tile();
    let mut pf = PathFinder::new();
    let path = pf.find(&w.cost, a, b, 3, 400000, crate::path::UNIT_WEIGHT);
    let end = path.last().copied().unwrap();
    assert!((end.0 as i32 - b.0).abs().max((end.1 as i32 - b.1).abs()) <= 3, "ボスポラスを渡れない");

    let w = World::new(12345, Scen::Second);
    for c in ["ダマスカス", "アンティオキア", "コンスタンティノープル", "カイロ", "バグダード"] {
        assert!(w.same_land(city_pos(&w, "エルサレム"), city_pos(&w, c)), "エルサレムから{}へ行けない", c);
    }
    let w = World::new(12345, Scen::Third);
    for c in ["ティルス", "トリポリ", "ヤッファ", "アスカロン", "カイロ", "アンティオキア", "ダマスカス"] {
        assert!(w.same_land(city_pos(&w, "アッコン"), city_pos(&w, c)), "アッコンから{}へ行けない", c);
    }
    assert!(!w.same_land(city_pos(&w, "アッコン"), city_pos(&w, "リマソル")), "キプロスが陸続き");
}

/// 街は築けない。砦は築ける。船を造れるのはイスラームの沿岸国家・イタリア・ビザンツだけ
#[test]
fn building_rules() {
    let mut w = World::new(12345, Scen::Second);
    let n = ZENGID;
    w.nations[n].res = [99999.0; 3];
    let home = w.nations[n].home;
    let mut town_ok = false;
    let mut fort_ok = false;
    for r in 3..10 {
        for dy in -r..=r {
            for dx in -r..=r {
                let (tx, ty) = (home.x as i32 + dx, home.y as i32 + dy);
                town_ok |= w.can_place(n, BKind::Town, tx, ty).is_ok();
                fort_ok |= w.can_place(n, BKind::Fort, tx, ty).is_ok();
            }
        }
    }
    assert!(!town_ok, "街を築けてしまう");
    assert!(fort_ok, "砦を築けない");
    for k in 0..NATIONS.len() {
        let ok = w.can_build_ships(k);
        assert_eq!(ok, [FATIMID, AYYUBID, BYZANTIUM, VENICE, GENOA, PISA].contains(&k), "{}", NATIONS[k].name);
    }
    // ファーティマ朝の都市を奪っても、十字軍は船を造れない
    let asc = w.city_bld("アスカロン").unwrap();
    w.blds[asc].nation = CRUSADERS as u8;
    assert!(!w.can_build_ships(CRUSADERS));
    assert!(w.available_units(CRUSADERS).iter().all(|&(u, _)| udef(u).class != UnitKind::Ship));
}

/// 第1回の十字軍は兵も民も補充できない
#[test]
fn first_crusade_no_reinforcements() {
    let mut w = World::new(12345, Scen::First);
    w.player = CRUSADERS;
    w.nations[CRUSADERS].res = [99999.0; 3];
    // アンティオキアを奪っても訓練できない
    let i = w.city_bld("アンティオキア").unwrap();
    w.blds[i].nation = CRUSADERS as u8;
    let h = w.bh(i);
    assert!(w.trainable_at(&w.blds[i]).is_empty());
    assert!(w.enqueue(h, UT_WORKER).is_err());
    // 第2回の十字軍は訓練できる
    let mut w = World::new(12345, Scen::Second);
    let i = w.city_bld("エルサレム").unwrap();
    let h = w.bh(i);
    w.nations[CRUSADERS].res = [99999.0; 3];
    assert!(w.enqueue(h, UT_WORKER).is_ok());
}

/// 進軍で疲れる。ビザンツ領では騎士だけが癒え、民衆十字軍は癒えない。占領地では癒える。民家を焼けば癒える
#[test]
fn crusade_attrition_and_recovery() {
    let mut w = World::new(12345, Scen::First);
    w.player = FATIMID;
    let byz = city_pos(&w, "アドリアノープル");
    let knight = w.spawn_unit(CRUSADERS, UT_KNIGHT, byz + v2(1.5, 1.5), 1.0);
    let people = w.spawn_unit(CRUSADERS, UT_PEOPLE, byz + v2(-1.5, 1.5), 1.0);
    for h in [knight, people] {
        let u = w.unit_mut(h).unwrap();
        u.hp = u.max_hp() * 0.5;
    }
    let (x, y) = (byz + v2(1.5, 1.5)).tile();
    assert_eq!(w.owner[Map::idx(x, y)] as usize, BYZANTIUM);
    for _ in 0..300 {
        w.crusade_attrition(DT);
    }
    let k = w.unit(knight).unwrap();
    assert!(k.hp > k.max_hp() * 0.5, "ビザンツ領で騎士が癒えない");
    let p = w.unit(people).unwrap();
    assert!(p.hp <= p.max_hp() * 0.5, "民衆十字軍がビザンツ領で癒えた");
    // 敵地を進むと疲れる
    let (x, y) = ll_to_tile(33.5, 38.6);
    let m = w.spawn_unit(CRUSADERS, UT_SERGEANT, w.passable_at_or_near(v2(x, y)), 1.0);
    w.unit_mut(m).unwrap().order = Order::Move(v2(x + 30.0, y));
    let hp0 = w.unit(m).unwrap().hp;
    for _ in 0..600 {
        w.crusade_attrition(DT);
    }
    assert!(w.unit(m).unwrap().hp < hp0, "進軍で疲れない");
    // 民家を焼くと近くの兵が癒える
    let vi = (0..w.blds.len()).find(|&i| w.blds[i].alive && w.blds[i].kind == BKind::Village && w.at_war(CRUSADERS, w.blds[i].nation as usize)).expect("敵の民家が無い");
    let c = w.blds[vi].center();
    let raider = w.spawn_unit(CRUSADERS, UT_KNIGHT, w.passable_at_or_near(c + v2(1.5, 0.0)), 1.0);
    w.unit_mut(raider).unwrap().hp = 30.0;
    let res0 = w.nations[CRUSADERS].res[0];
    w.damage_bld(vi, 9999.0, CRUSADERS);
    assert!(!w.blds[vi].alive || w.blds[vi].kind != BKind::Village);
    assert!(w.unit(raider).unwrap().hp > 30.0 + 40.0, "民家を焼いても癒えない");
    assert!(w.nations[CRUSADERS].res[0] > res0);
    assert_eq!(w.ruins.len(), 1);
}

fn run_checks(w: &mut World, secs: f32) {
    for _ in 0..(secs / DT) as usize {
        w.scenario_tick(DT);
        if w.game_over.is_some() {
            break;
        }
    }
}

/// 第1回：エルサレムを奪って5年守れば十字軍の勝ち。期限までに奪えなければイスラーム勢力の勝ち
#[test]
fn first_crusade_victory() {
    // 十字軍として：奪って5年
    let mut w = World::new(12345, Scen::First);
    w.player = CRUSADERS;
    let j = w.city_bld("エルサレム").unwrap();
    w.years = HIST_JERUSALEM - w.start_ad() - 0.2;
    w.blds[j].nation = CRUSADERS as u8;
    run_checks(&mut w, 1.0);
    assert!(w.st.jeru_since.is_some());
    assert!(w.game_over.is_none());
    w.years += 4.9;
    run_checks(&mut w, 1.0);
    assert!(w.game_over.is_none());
    w.years += 0.2;
    run_checks(&mut w, 1.0);
    assert!(matches!(&w.game_over, Some((true, _))), "{:?}", w.game_over);
    // ファーティマ朝として：取り返せば勝ち
    let mut w = World::new(12345, Scen::First);
    w.player = FATIMID;
    let j = w.city_bld("エルサレム").unwrap();
    w.blds[j].nation = CRUSADERS as u8;
    run_checks(&mut w, 1.0);
    assert!(w.game_over.is_none());
    w.blds[j].nation = FATIMID as u8;
    run_checks(&mut w, 1.0);
    assert!(matches!(&w.game_over, Some((true, _))), "{:?}", w.game_over);
    // 期限切れ
    let mut w = World::new(12345, Scen::First);
    w.player = CRUSADERS;
    w.years = HIST_JERUSALEM - w.start_ad() + 0.01;
    run_checks(&mut w, 1.0);
    assert!(matches!(&w.game_over, Some((false, _))), "{:?}", w.game_over);
    // 全滅
    let mut w = World::new(12345, Scen::First);
    w.player = FATIMID;
    for i in 0..w.units.len() {
        if w.units[i].alive && w.units[i].nation as usize == CRUSADERS {
            w.kill_unit(i);
        }
    }
    run_checks(&mut w, 1.0);
    assert!(!w.nations[CRUSADERS].alive);
    assert!(matches!(&w.game_over, Some((true, _))), "{:?}", w.game_over);
}

/// 第2回：十字軍はダマスカスを奪えば勝ち。ほかの国は1250年の版図で競う
#[test]
fn second_crusade_victory() {
    let mut w = World::new(12345, Scen::Second);
    w.player = CRUSADERS;
    let d = w.city_bld("ダマスカス").unwrap();
    w.blds[d].nation = CRUSADERS as u8;
    run_checks(&mut w, 1.0);
    assert!(matches!(&w.game_over, Some((true, _))), "{:?}", w.game_over);
    let mut w = World::new(12345, Scen::Second);
    w.player = ARMENIA;
    w.years = w.end_ad() - w.start_ad() + 0.01;
    w.update_stats();
    run_checks(&mut w, 1.0);
    assert!(matches!(&w.game_over, Some((false, _))), "小国が版図で勝ってしまう {:?}", w.game_over);
    let mut w = World::new(12345, Scen::Second);
    w.player = BYZANTIUM;
    w.years = w.end_ad() - w.start_ad() + 0.01;
    w.recompute_territory();
    w.update_stats();
    let best = (0..NATIONS.len()).filter(|&n| w.nations[n].alive).max_by_key(|&n| w.nations[n].tiles).unwrap();
    w.player = best;
    run_checks(&mut w, 1.0);
    assert!(matches!(&w.game_over, Some((true, _))), "{} {:?}", NATIONS[best].name, w.game_over);
}

/// 第3回：沿岸の都市から相手を駆逐すれば勝ち。1192年9月まで決まらなければ和約
#[test]
fn third_crusade_victory() {
    let mut w = World::new(12345, Scen::Third);
    w.player = CRUSADERS;
    for b in w.blds.iter_mut() {
        if b.alive && b.kind.city() && cities()[b.site as usize].coast {
            b.nation = CRUSADERS as u8;
        }
    }
    run_checks(&mut w, 1.0);
    assert!(matches!(&w.game_over, Some((true, _))), "{:?}", w.game_over);
    let mut w = World::new(12345, Scen::Third);
    w.player = CRUSADERS;
    for b in w.blds.iter_mut() {
        if b.alive && b.kind.city() && cities()[b.site as usize].coast {
            b.nation = AYYUBID as u8;
        }
    }
    run_checks(&mut w, 1.0);
    assert!(matches!(&w.game_over, Some((false, _))) && !w.draw, "{:?}", w.game_over);
    let mut w = World::new(12345, Scen::Third);
    w.player = AYYUBID;
    w.years = w.end_ad() - w.start_ad() + 0.01;
    run_checks(&mut w, 1.0);
    assert!(w.draw && matches!(&w.game_over, Some((false, _))), "{:?}", w.game_over);
}

/// 史実の出来事：1098年8月にファーティマ朝がアルトゥク家からエルサレムを奪う。王朝の交代で国の名が変わる
#[test]
fn history_events() {
    for w2 in HISTORY.windows(2) {
        if w2[0].scen == w2[1].scen {
            assert!(w2[0].ad <= w2[1].ad, "{} の後に {}", w2[0].title, w2[1].title);
        }
    }
    for e in HISTORY {
        let d = e.scen.def();
        assert!(e.ad >= d.start_ad && e.ad <= d.end_ad, "{}", e.title);
        assert!(present(e.scen, e.nation), "{}：{} が登場しない", e.title, NATIONS[e.nation].name);
        set_scen(e.scen);
        for fx in e.fx {
            match *fx {
                Fx::Seize(c, from) => {
                    assert!(city_index(c).is_some(), "{}：{} が無い", e.title, c);
                    assert!(present(e.scen, from));
                }
                Fx::Leave(h, _) => assert!(HEROES.iter().any(|x| x.name == h && x.scen == e.scen), "{}：{}", e.title, h),
                _ => {}
            }
        }
    }
    let mut w = World::new(12345, Scen::First);
    w.player = CRUSADERS;
    w.years = 1098.65 - w.start_ad();
    w.run_history();
    let j = w.city_bld("エルサレム").unwrap();
    assert_eq!(w.blds[j].nation as usize, FATIMID);
    let mut w = World::new(12345, Scen::Second);
    w.player = BYZANTIUM;
    w.years = 1172.0 - w.start_ad();
    w.run_history();
    assert_eq!(w.nations[FATIMID].name, "アイユーブ朝");
}

/// 全ての国を AI に任せて回す（SIM_SCEN=1〜3、SIM_YEARS=年数）
#[test]
fn long_sim() {
    let sc = std::env::var("SIM_SCEN").ok().and_then(|s| s.parse::<u8>().ok()).unwrap_or(2);
    let s = Scen::from_u8(sc.saturating_sub(1));
    let seed = std::env::var("SIM_SEED").ok().and_then(|s| s.parse().ok()).unwrap_or(12345u32);
    let mut w = World::new(seed, s);
    let pk = std::env::var("SIM_PICK").ok().and_then(|s| s.parse::<usize>().ok()).unwrap_or(0);
    w.player = s.def().picks[pk.min(s.def().picks.len() - 1)].nation;
    w.ai_player = true;
    let t0 = std::time::Instant::now();
    let years = std::env::var("SIM_YEARS").ok().and_then(|s| s.parse().ok()).unwrap_or(w.end_ad() - w.start_ad());
    let report = (years / 10.0).max(0.05);
    let mut next_report = 0.0;
    let mut seen_t = -1.0f32;
    let mut steps = 0usize;
    while w.years < years {
        steps += 1;
        w.step();
        if w.years >= next_report {
            next_report += report;
            let alive: Vec<String> = w
                .nations
                .iter()
                .filter(|n| n.alive)
                .map(|n| format!("{}(c{} w{} s{} t{} 糧{:.0} 木{:.0} 鉄{:.0})", n.name, n.cities, n.workers, n.soldiers, n.tiles, n.res[0], n.res[1], n.res[2]))
                .collect();
            let units = w.units.iter().filter(|u| u.alive).count();
            println!("{} t={:.1}s units={} :: {}", w.date_str(), t0.elapsed().as_secs_f32(), units, alive.join(" "));
        }
        for m in w.msgs.iter().filter(|m| m.time > seen_t) {
            println!("  [msg] {}", m.text);
        }
        seen_t = w.time;
        if let Some((win, t)) = &w.game_over {
            println!("終了：{} {} {}", win, w.draw, t);
            break;
        }
    }
    println!("elapsed {:.1}s for {} steps", t0.elapsed().as_secs_f32(), steps);
}

