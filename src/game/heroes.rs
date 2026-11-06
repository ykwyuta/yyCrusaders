//! 英雄の一覧と、地図上の英雄の名札

use super::*;

impl Game {
    /// 英雄の一覧：この遠征に登場する実在の人物と、いまの居場所
    pub(super) fn render_summon(&mut self) {
        let (w, h) = (self.gfx.width as f32, self.gfx.height as f32);
        let pl = self.world.player;
        let pw = 1120.0f32.min(w - 20.0);
        let ph = 680.0f32.min(h - 30.0);
        let x0 = (w - pw) / 2.0;
        let y0 = (h - ph) / 2.0;
        self.gfx.fill_rect(0.0, 0.0, w, h, [0.0, 0.0, 0.0, 0.5]);
        self.art.draw_stone_panel(&self.gfx, x0, y0, x0 + pw, y0 + ph);
        self.art.draw_corner_ornaments(&self.gfx, x0 + 3.0, y0 + 3.0, x0 + pw - 3.0, y0 + ph - 3.0, 16.0);
        self.gfx.text_shadow(t!("英雄", "Heroes", "英雄"), x0 + 24.0, y0 + 10.0, 160.0, 34.0, Font::Big, [1.0, 0.85, 0.5, 1.0]);
        self.add_button(x0 + pw - 90.0, y0 + 10.0, x0 + pw - 14.0, y0 + 40.0, t!("閉じる", "Close", "关闭"), "", 0, true, Act::CloseOverlay, "");
        let gold = [1.0, 0.85, 0.5, 1.0];
        let tc = [0.95, 0.92, 0.85, 1.0];
        let dim = [0.72, 0.69, 0.62, 1.0];
        let head = t!("この遠征のときに実在した人物が、仕えていた国の兵として戦う。英雄は傷が癒えるが、討たれれば二度と戻らない。自国の英雄の「見る」で居場所へ移る", "Real people of this crusade fight for the states they served. Heroes heal over time, but once slain they never return. Press \"View\" on one of your heroes to jump to them", "本次远征时真实存在的人物，作为其效力之国的士兵作战。英雄会恢复伤势，但一旦战死便不再归来。点击己方英雄的“查看”即可前往其所在");
        self.gfx.text(head, x0 + 150.0, y0 + 12.0, pw - 260.0, 34.0, Font::Small, tc);

        self.art.draw_inset_box(&self.gfx, x0 + 10.0, y0 + 48.0, x0 + pw - 10.0, y0 + ph - 10.0);
        let s = self.world.scen;
        let list: Vec<usize> = (0..HEROES.len()).filter(|&k| HEROES[k].scen == s).collect();
        let cols = 2;
        let per = (list.len() + 1) / cols;
        let colw = (pw - 30.0) / cols as f32;
        let rowh = ((ph - 70.0) / per.max(1) as f32).min(56.0);
        for (j, &k) in list.iter().enumerate() {
            let d = &HEROES[k];
            let c = j / per.max(1);
            let r = j % per.max(1);
            let rx = x0 + 15.0 + c as f32 * colw;
            let ry = y0 + 54.0 + r as f32 * rowh;
            if r % 2 == 0 {
                self.gfx.fill_rect(rx, ry, rx + colw - 8.0, ry + rowh, [1.0, 1.0, 1.0, 0.04]);
            }
            let unit = (0..self.world.units.len()).find(|&i| self.world.units[i].alive && self.world.units[i].hero as usize == k);
            let isz = (rowh - 4.0).clamp(24.0, 48.0);
            let ix0 = rx + 4.0;
            let iy0 = ry + (rowh - isz) * 0.5;
            self.gfx.fill_round(ix0, iy0, ix0 + isz, iy0 + isz, 3.0, [0.035, 0.085, 0.12, 0.96]);
            if let Some(hbmp) = self.art.period_hero(k, false) {
                self.gfx.draw_bitmap(hbmp, ix0 + 1.0, iy0 + 1.0, ix0 + isz - 1.0, iy0 + isz - 1.0);
            }
            let nc = self.world.nations[d.nation].color;
            self.gfx.stroke_round(ix0, iy0, ix0 + isz, iy0 + isz, 3.0, 1.6, [nc[0], nc[1], nc[2], 0.95]);
            let tx = rx + isz + 10.0;
            let ny = ry + (rowh - 38.0).max(0.0) * 0.5;
            let alive = unit.is_some();
            self.gfx.text(crate::lang::tx(d.name), tx, ny, 250.0, 20.0, Font::Normal, if alive { gold } else { dim });
            self.gfx.text(crate::lang::tx(d.era), tx, ny + 19.0, 250.0, 17.0, Font::Small, dim);
            let st = match unit {
                Some(i) => {
                    let u = &self.world.units[i];
                    tf!("{}・{}　体力 {:.0}%", "{} · {}  HP {:.0}%", "{}・{}　体力 {:.0}%", crate::lang::tx(self.world.nations[u.nation as usize].name), crate::lang::tx(udef(u.ut).name), u.hp / u.max_hp() * 100.0)
                }
                None => tf!("{}・世を去った", "{} · gone", "{}・已不在", crate::lang::tx(self.world.nations[d.nation].name)),
            };
            self.gfx.text(&st, tx + 254.0, ry + (rowh - 20.0) * 0.5, colw - (isz + 10.0) - 340.0, 20.0, Font::Small, if alive { [0.7, 0.92, 1.0, 1.0] } else { [1.0, 0.6, 0.5, 1.0] });
            let own = unit.map_or(false, |i| self.world.units[i].nation as usize == pl);
            if own {
                self.add_button(rx + colw - 82.0, ry + 3.0, rx + colw - 14.0, ry + rowh - 3.0, t!("見る", "View", "查看"), "", 0, true, Act::SummonHero(k), crate::lang::tx(d.desc));
            } else {
                self.buttons.push(Button { x0: rx, y0: ry, x1: rx + colw - 90.0, y1: ry + rowh, label: String::new(), key: 0, enabled: true, act: Act::Noop, tip: crate::lang::tx(d.desc).to_string() });
            }
        }
    }

    /// 都市の名札（実在の街の名。第3回は勝敗に関わる沿岸の都市に錨印）
    pub(super) fn render_city_labels(&mut self) {
        let (w, h) = (self.gfx.width as f32, self.gfx.height as f32);
        let z = self.zoom;
        if z < 1.6 {
            return;
        }
        let third = self.world.scen == Scen::Third;
        let mut labels = Vec::new();
        for b in self.world.blds.iter() {
            if !b.alive || !b.kind.city() || b.site == u16::MAX {
                continue;
            }
            let c = b.center();
            if !self.explored(c) {
                continue;
            }
            let Some(cd) = cities().get(b.site as usize) else { continue };
            let coast = third && cd.coast;
            // 遠くから見ているときは都と沿岸の都市だけ
            if z < 4.0 && b.kind != BKind::Capital && !coast {
                continue;
            }
            let (sx, sy) = self.world_to_screen(c);
            if sx < -80.0 || sy < -40.0 || sx > w + 80.0 || sy > h + 40.0 {
                continue;
            }
            let name = crate::lang::tx(cd.name);
            let text = if coast { format!("⚓{}", name) } else { name.to_string() };
            labels.push((sx, sy, text, self.world.nations[b.nation as usize].color, b.kind == BKind::Capital));
        }
        for (sx, sy, text, c, cap) in labels {
            let ty = sy + (z * if cap { 1.25 } else { 1.0 }).clamp(8.0, 46.0);
            let hw = self.gfx.measure(&text, Font::NormalCenter) * 0.5 + 6.0;
            self.gfx.fill_round(sx - hw, ty, sx + hw, ty + 19.0, 3.0, [0.05, 0.05, 0.08, 0.72]);
            self.gfx.fill_rect(sx - hw, ty + 16.0, sx + hw, ty + 19.0, [c[0], c[1], c[2], 0.95]);
            self.gfx.text(&text, sx - hw, ty + 1.0, hw * 2.0, 17.0, Font::NormalCenter, if cap { [1.0, 0.9, 0.55, 1.0] } else { [0.95, 0.93, 0.88, 1.0] });
        }
    }

    /// 英雄の名札（地図上）
    pub(super) fn render_hero_labels(&mut self) {
        let (w, h) = (self.gfx.width as f32, self.gfx.height as f32);
        let pl = self.world.player;
        let z = self.zoom;
        let mut labels = Vec::new();
        for u in self.world.units.iter() {
            if !u.alive || (u.hero as usize) >= HEROES.len() || (u.nation as usize != pl && !self.seen(u.pos)) {
                continue;
            }
            let (sx, sy) = self.world_to_screen(u.pos);
            if sx < -80.0 || sy < -40.0 || sx > w + 80.0 || sy > h + 40.0 {
                continue;
            }
            labels.push((sx, sy, crate::lang::tx(HEROES[u.hero as usize].name), self.world.nations[u.nation as usize].color));
        }
        for (sx, sy, name, c) in labels {
            let ty = sy - (z * 1.3).clamp(14.0, 34.0) - 22.0;
            let hw = (self.gfx.measure(name, Font::NormalCenter) * 0.5 + 8.0).max(40.0);
            self.gfx.fill_round(sx - hw, ty, sx + hw, ty + 20.0, 4.0, [0.10, 0.07, 0.02, 0.85]);
            self.gfx.stroke_round(sx - hw, ty, sx + hw, ty + 20.0, 4.0, 1.4, [c[0] * 0.5 + 0.5, c[1] * 0.5 + 0.5, c[2] * 0.5 + 0.5, 0.95]);
            self.gfx.text(name, sx - hw, ty + 2.0, hw * 2.0, 18.0, Font::NormalCenter, [1.0, 0.92, 0.6, 1.0]);
        }
    }
}
