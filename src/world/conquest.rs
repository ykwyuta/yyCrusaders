//! 征服にまつわる出来事：邑の破壊（焼き払い）、服属（併合）、史実の出来事による都市の奪取、
//! 王朝の交代（ファーティマ朝からアイユーブ朝へ）、英雄の帰国・暗殺

use super::*;

impl World {
    /// 邑を破壊する（焼き払う）。都は破壊できない。戦利品として資源を得る
    pub fn can_raze(&self, bi: usize, by: usize) -> Result<(), &'static str> {
        let b = &self.blds[bi];
        if !b.alive || !b.kind.city() || b.nation as usize != by {
            return Err(t!("自国の邑だけを破壊できる", "You can only raze your own towns", "只能摧毁己方的邑"));
        }
        if b.kind == BKind::Capital {
            return Err(t!("都は破壊できない", "The capital cannot be razed", "都城无法摧毁"));
        }
        if b.site != u16::MAX {
            return Err(t!("実在の都市は焼き払えない（街は新しく築けない）", "Historical cities cannot be razed (no new cities can be founded)", "真实城市无法焚毁（无法新建城市）"));
        }
        if !b.done() {
            return Err(t!("建設中の邑は取り壊す", "Demolish a town still under construction instead", "建造中的邑请直接拆除"));
        }
        Ok(())
    }

    /// 邑を焼き払う。戦利品（糧・木・鉄）を返す
    pub fn raze_city(&mut self, bi: usize, by: usize) -> [f32; 3] {
        if self.can_raze(bi, by).is_err() {
            return [0.0; 3];
        }
        let origin = self.blds[bi].origin as usize;
        let place = self.city_place(bi);
        let m = self.era_mult();
        // 他国が築いた邑ほど多く奪える
        let k = if origin != by { 1.0 } else { 0.5 };
        let loot = [160.0 * m * k, 110.0 * m * k, 70.0 * m * k];
        self.remove_building(bi);
        if by < NATIONS.len() {
            let n = &mut self.nations[by];
            for r in 0..3 {
                n.res[r] += loot[r];
            }
            n.aggression += 15.0;
        }
        let who = tx(self.nations[by].name);
        let text = tf!("{} が{}の邑を焼き払った（糧{:.0}・木{:.0}・鉄{:.0}を得た）", "{} razed the town at {} (took food {:.0}, wood {:.0}, iron {:.0})", "{}焚毁了{}的邑（获得粮{:.0}、木{:.0}、铁{:.0}）", who, place, loot[0], loot[1], loot[2]);
        let col = if by == self.player { [1.0, 0.75, 0.4] } else { [0.85, 0.7, 0.6] };
        self.push_msg(text, col);
        self.territory_dirty = true;
        loot
    }

    /// AI：攻め落とした邑を焼き払うか（遊牧の国ほど焼く。城邑が少なければ焼かない）
    pub(super) fn ai_wants_raze(&mut self, an: usize, bi: usize) -> bool {
        let cities = self.blds.iter().filter(|b| b.alive && b.nation as usize == an && b.kind.city() && b.done()).count();
        if cities < 4 || self.blds[bi].origin as usize == an {
            return false;
        }
        // 実在の都市は焼かない（街は新しく築けないので、焼けば二度と戻らない）
        if self.blds[bi].site != u16::MAX {
            return false;
        }
        let p = 0.04;
        self.rng.chance(p)
    }

    /// 服属（併合）：target の城邑と兵が by のものになる。プレイヤーの国は併合されず、攻め込まれる
    pub(super) fn annex(&mut self, by: usize, target: usize) -> Option<String> {
        if by == target || !self.nations[by].alive {
            return None;
        }
        if !self.nations[target].alive {
            // 既に滅んでいれば、散り散りの遺民が馳せ参じる
            let Some(c) = self.nations[by].capital else { return None };
            let ut = best_land_ut(by);
            let pw = (1.0 + self.years / (self.end_ad() - self.start_ad()) * 2.0).min(self.power_cap());
            let mut k = 0;
            for _ in 0..3 {
                if let Some(sp) = self.spawn_point(c) {
                    self.spawn_unit(by, ut, sp, pw);
                    k += 1;
                }
            }
            let r = &mut self.nations[by].res;
            r[0] += 300.0;
            r[1] += 150.0;
            r[2] += 100.0;
            self.nations[by].cap_bonus += 2;
            return Some(tf!("{}の遺民が{}に加わった（兵{}人、糧300・木150・鉄100）", "Survivors of the {} join the {} ({} troops, food 300, wood 150, iron 100)", "{}遗民归附{}（士兵{}人，粮300、木150、铁100）", tx(self.nations[target].name), tx(self.nations[by].name), k));
        }
        if target == self.player && !self.ai_player {
            if self.declare_war(by, target, true) {
                return Some(tf!("{}は服属を拒み、{}が攻め込む", "{} refuses to submit, and {} invades", "{}拒绝臣服，{}发兵来攻", tx(self.nations[target].name), tx(self.nations[by].name)));
            }
            return None;
        }
        for i in 0..self.blds.len() {
            if self.blds[i].alive && self.blds[i].nation as usize == target {
                self.blds[i].nation = by as u8;
                self.blds[i].queue.clear();
                if self.blds[i].kind == BKind::Capital {
                    self.blds[i].kind = BKind::Town;
                }
            }
        }
        let mut k = 0;
        for i in 0..self.units.len() {
            if self.units[i].alive && self.units[i].nation as usize == target {
                self.units[i].nation = by as u8;
                self.units[i].order = Order::Idle;
                self.units[i].build_queue.clear();
                k += 1;
            }
        }
        let res = self.nations[target].res;
        for r in 0..3 {
            self.nations[by].res[r] += res[r];
        }
        let bonus = self.nations[target].cap_bonus;
        self.nations[by].cap_bonus += bonus + BASE_CAP;
        let t = &mut self.nations[target];
        t.alive = false;
        t.capital = None;
        t.cap_bonus = 0;
        for o in 0..NATIONS.len() {
            self.war[target][o] = false;
            self.war[o][target] = false;
        }
        self.coalitions.retain(|c| c.target != target);
        for c in self.coalitions.iter_mut() {
            c.members.retain(|&m| m != target);
        }
        if self.nations[by].capital.and_then(|h| self.bld(h)).is_none() {
            if let Some(i) = (0..self.blds.len()).find(|&i| self.blds[i].alive && self.blds[i].nation as usize == by && self.blds[i].kind.city()) {
                self.blds[i].kind = BKind::Capital;
                let h = self.bh(i);
                self.nations[by].capital = Some(h);
                self.nations[by].home = self.blds[i].center();
            }
        }
        if target == self.player {
            self.player = by;
        }
        self.territory_dirty = true;
        self.update_stats();
        Some(tf!("{}が{}に服属した（兵{}人と城邑が加わる）", "{} submits to {} ({} troops and its cities join)", "{}臣服于{}（{}名士兵与城邑归附）", tx(self.nations[target].name), tx(self.nations[by].name), k))
    }

    /// 史実の出来事で、名前の都市を from から by へ移す（プレイヤーの都市は奪わず、攻め込む）
    pub(super) fn seize_city(&mut self, by: usize, name: &str, from: usize) -> Option<String> {
        let bi = self.city_bld(name)?;
        let old = self.blds[bi].nation as usize;
        if old != from || old == by || !self.nations[by].alive {
            return None;
        }
        if old == self.player && !self.ai_player {
            if self.declare_war(by, old, true) {
                return Some(tf!("{}は{}を求めて{}に攻め込む", "{} invades {} to claim {}", "{}为夺取{}进攻{}", tx(self.nations[by].name), tx(self.nations[old].name), tx(cities()[self.blds[bi].site as usize].name)));
            }
            return None;
        }
        let was_cap = self.blds[bi].kind == BKind::Capital;
        {
            let b = &mut self.blds[bi];
            b.nation = by as u8;
            b.queue.clear();
            b.rally = None;
            b.kind = BKind::Town;
            b.hp = b.hp.max(b.kind.max_hp() * 0.5);
        }
        // 都市の守備兵は退く（周りにいる旧主の兵は都市の外へ出る）
        let c = self.blds[bi].center();
        for i in 0..self.units.len() {
            if self.units[i].alive && self.units[i].nation as usize == old && self.units[i].pos.dist(c) < 5.0 && !self.units[i].kind.naval() {
                self.units[i].order = Order::Idle;
            }
        }
        if self.nations[by].capital.and_then(|h| self.bld(h)).is_none() {
            self.blds[bi].kind = BKind::Capital;
            let h = self.bh(bi);
            self.nations[by].capital = Some(h);
            self.nations[by].home = c;
        }
        if was_cap {
            let next = (0..self.blds.len()).find(|&i| self.blds[i].alive && self.blds[i].nation as usize == old && self.blds[i].kind == BKind::Town && self.blds[i].done());
            match next {
                Some(i) => {
                    self.blds[i].kind = BKind::Capital;
                    let h = self.bh(i);
                    self.nations[old].capital = Some(h);
                    self.nations[old].home = self.blds[i].center();
                }
                None => self.nations[old].capital = None,
            }
        }
        self.territory_dirty = true;
        let has_city = self.blds.iter().any(|b| b.alive && b.nation as usize == old && b.kind.city() && b.done());
        if !has_city && !self.army_nation(old) {
            self.eliminate(old, by);
        }
        Some(tf!("{}が{}のものとなる", "{} passes to {}", "{}归属{}", tx(name_static(name)), tx(self.nations[by].name)))
    }

    /// 王朝の交代：国の名と色が別の国（NATIONS の other）のものになる
    pub(super) fn become_nation(&mut self, n: usize, other: usize) -> Option<String> {
        if !self.nations[n].alive || self.nations[n].name == NATIONS[other].name {
            return None;
        }
        let old = self.nations[n].name;
        self.nations[n].name = NATIONS[other].name;
        self.nations[n].color = NATIONS[other].color;
        self.territory_version += 1;
        Some(tf!("{}は{}となった", "{} becomes {}", "{}成为{}", tx(old), tx(NATIONS[other].name)))
    }

    /// 英雄が戦場を去る（帰国・暗殺・病）
    pub(super) fn hero_leave(&mut self, name: &str, why: &'static str) -> Option<String> {
        let k = HEROES.iter().position(|h| h.name == name)?;
        let i = (0..self.units.len()).find(|&i| self.units[i].alive && self.units[i].hero as usize == k)?;
        self.kill_unit(i);
        Some(format!("{}{}", tx(HEROES[k].name), tx(why)))
    }
}

/// 都市の名前を、データの &'static str に揃える（訳を引くため）
fn name_static(name: &str) -> &'static str {
    cities().iter().find(|c| c.name == name).map_or("", |c| c.name)
}
