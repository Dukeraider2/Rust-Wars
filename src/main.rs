use macroquad::prelude::*;

const MAP_W: usize = 10;
const MAP_H: usize = 8;
const TILE: f32 = 64.0;
const BOARD_X: f32 = 38.0;
const BOARD_Y: f32 = 150.0;
const HUMAN: u8 = 0;
const BOTS: u8 = 1;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Terrain { Plain, Forest, Mountain }

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind { Infantry, Tank, Artillery }

#[derive(Clone)]
struct Unit {
    id: usize,
    team: u8,
    kind: Kind,
    x: usize,
    y: usize,
    hp: i32,
    acted: bool,
}

#[derive(Clone, Copy)]
struct Site { x: usize, y: usize, owner: Option<u8> }

struct Game {
    terrain: [[Terrain; MAP_W]; MAP_H],
    units: Vec<Unit>,
    sites: Vec<Site>,
    selected: Option<usize>,
    round: usize,
    over: Option<u8>,
    log: Vec<String>,
}

impl Game {
    fn new() -> Self {
        let mut terrain = [[Terrain::Plain; MAP_W]; MAP_H];
        for (x, y) in [(2,1),(3,1),(7,1),(1,3),(5,3),(8,3),(3,5),(6,6),(8,6)] {
            terrain[y][x] = Terrain::Forest;
        }
        for (x, y) in [(4,2),(5,2),(4,5),(5,5),(6,5)] { terrain[y][x] = Terrain::Mountain; }
        let units = vec![
            Unit { id: 1, team: HUMAN, kind: Kind::Infantry, x: 1, y: 6, hp: 10, acted: false },
            Unit { id: 2, team: HUMAN, kind: Kind::Tank, x: 2, y: 7, hp: 10, acted: false },
            Unit { id: 3, team: HUMAN, kind: Kind::Artillery, x: 0, y: 7, hp: 10, acted: false },
            Unit { id: 4, team: BOTS, kind: Kind::Infantry, x: 8, y: 1, hp: 10, acted: false },
            Unit { id: 5, team: BOTS, kind: Kind::Tank, x: 8, y: 0, hp: 10, acted: false },
            Unit { id: 6, team: BOTS, kind: Kind::Artillery, x: 9, y: 0, hp: 10, acted: false },
        ];
        let sites = vec![
            Site { x: 1, y: 2, owner: None }, Site { x: 4, y: 3, owner: None },
            Site { x: 7, y: 4, owner: None }, Site { x: 8, y: 7, owner: None },
        ];
        Self {
            terrain, units, sites, selected: None, round: 1, over: None,
            log: vec!["FRONTIER 09 // capture three command sites".to_string(), "Select a unit, then click a highlighted tile.".to_string()],
        }
    }

    fn update(&mut self) {
        if self.over.is_some() {
            if is_key_pressed(KeyCode::Enter) || is_mouse_button_pressed(MouseButton::Left) { *self = Self::new(); }
            return;
        }
        if is_key_pressed(KeyCode::Escape) { self.selected = None; }
        if is_key_pressed(KeyCode::E) { self.end_turn(); }
        if is_mouse_button_pressed(MouseButton::Left) {
            let (mx, my) = mouse_position();
            if Rect::new(1020.0, 710.0, 212.0, 48.0).contains(vec2(mx, my)) { self.end_turn(); }
            else if mx >= BOARD_X && mx < BOARD_X + MAP_W as f32 * TILE && my >= BOARD_Y && my < BOARD_Y + MAP_H as f32 * TILE {
                let x = ((mx - BOARD_X) / TILE) as usize;
                let y = ((my - BOARD_Y) / TILE) as usize;
                self.click_tile(x, y);
            } else if mx < 690.0 { self.selected = None; }
        }
        if is_mouse_button_pressed(MouseButton::Right) { self.selected = None; }
    }

    fn click_tile(&mut self, x: usize, y: usize) {
        if let Some(i) = self.units.iter().position(|u| u.x == x && u.y == y) {
            let unit = self.units[i].clone();
            if unit.team == HUMAN {
                self.selected = Some(unit.id);
                self.note(format!("{} selected // {} HP", kind_name(unit.kind), unit.hp));
            } else if let Some(id) = self.selected {
                self.attack(id, unit.id);
            }
            return;
        }
        if let Some(id) = self.selected { self.move_unit(id, x, y); }
    }

    fn move_unit(&mut self, id: usize, x: usize, y: usize) {
        let Some(i) = self.units.iter().position(|u| u.id == id && u.team == HUMAN) else { return; };
        let u = self.units[i].clone();
        if u.acted { self.note("Order already spent this round.".into()); return; }
        let reachable = self.reachable(i);
        if !reachable[y][x] || (u.x == x && u.y == y) { self.note("Tile is out of movement range.".into()); return; }
        self.units[i].x = x;
        self.units[i].y = y;
        self.units[i].acted = true;
        self.note(format!("{} advances to {},{}.", kind_name(u.kind), x + 1, y + 1));
        self.capture(x, y, HUMAN, u.kind);
        self.check_victory();
    }

    fn attack(&mut self, attacker_id: usize, target_id: usize) {
        let Some(ai) = self.units.iter().position(|u| u.id == attacker_id && u.team == HUMAN) else { return; };
        let Some(ti) = self.units.iter().position(|u| u.id == target_id && u.team == BOTS) else { return; };
        let a = self.units[ai].clone();
        let t = self.units[ti].clone();
        if a.acted { self.note("Order already spent this round.".into()); return; }
        if !can_attack(a.kind, a.x, a.y, t.x, t.y) { self.note("Target is outside weapon range.".into()); return; }
        self.deal_damage(ai, ti);
        self.units[ai].acted = true;
        self.check_victory();
    }

    fn deal_damage(&mut self, ai: usize, ti: usize) {
        let attacker = self.units[ai].clone();
        let target = self.units[ti].clone();
        let cover = if self.terrain[target.y][target.x] == Terrain::Forest { 1 } else { 0 };
        let amount = (damage(attacker.kind, target.kind) - cover).max(1);
        self.units[ti].hp -= amount;
        self.note(format!("{} hits {} for {} // {} HP left", kind_name(attacker.kind), kind_name(target.kind), amount, self.units[ti].hp.max(0)));
        if self.units[ti].hp <= 0 {
            self.note(format!("BOT UNIT {} DESTROYED", target.id));
            self.units.remove(ti);
        }
    }

    fn capture(&mut self, x: usize, y: usize, team: u8, kind: Kind) {
        if kind != Kind::Infantry { return; }
        let captured = if let Some(site) = self.sites.iter_mut().find(|s| s.x == x && s.y == y) {
            if site.owner != Some(team) { site.owner = Some(team); true } else { false }
        } else { false };
        if captured { self.note(if team == HUMAN { "COMMAND SITE SECURED // HUMAN CONTROL".into() } else { "COMMAND SITE LOST // BOT CONTROL".into() }); }
    }

    fn reachable(&self, unit_index: usize) -> [[bool; MAP_W]; MAP_H] {
        let unit = &self.units[unit_index];
        let allowance = move_range(unit.kind);
        let mut dist = [[i32::MAX; MAP_W]; MAP_H];
        dist[unit.y][unit.x] = 0;
        for _ in 0..MAP_W * MAP_H {
            let mut changed = false;
            for y in 0..MAP_H { for x in 0..MAP_W {
                if dist[y][x] == i32::MAX { continue; }
                for (nx, ny) in neighbors(x, y) {
                    if nx >= MAP_W || ny >= MAP_H || self.terrain[ny][nx] == Terrain::Mountain { continue; }
                    if self.units.iter().any(|other| other.id != unit.id && other.x == nx && other.y == ny) { continue; }
                    let cost = if self.terrain[ny][nx] == Terrain::Forest { 2 } else { 1 };
                    let next = dist[y][x] + cost;
                    if next < dist[ny][nx] && next <= allowance { dist[ny][nx] = next; changed = true; }
                }
            }}
            if !changed { break; }
        }
        let mut result = [[false; MAP_W]; MAP_H];
        for y in 0..MAP_H { for x in 0..MAP_W { result[y][x] = dist[y][x] <= allowance; }}
        result
    }

    fn end_turn(&mut self) {
        if self.over.is_some() { return; }
        self.selected = None;
        self.note(format!("ROUND {} // BOT RESPONSE", self.round));
        let ids: Vec<usize> = self.units.iter().filter(|u| u.team == BOTS).map(|u| u.id).collect();
        for id in ids {
            let Some(i) = self.units.iter().position(|u| u.id == id) else { continue; };
            self.bot_action(i);
            self.check_victory();
            if self.over.is_some() { return; }
        }
        self.round += 1;
        for unit in &mut self.units { unit.acted = false; }
        self.note(format!("ROUND {} // YOUR ORDERS", self.round));
    }

    fn bot_action(&mut self, i: usize) {
        let bot = self.units[i].clone();
        if let Some(t) = self.units.iter().find(|u| u.team == HUMAN && can_attack(bot.kind, bot.x, bot.y, u.x, u.y)).cloned() {
            if let Some(ti) = self.units.iter().position(|u| u.id == t.id) { self.bot_damage(i, ti); }
            return;
        }
        let Some(goal) = self.sites.iter().filter(|s| s.owner != Some(BOTS)).min_by_key(|s| manhattan(bot.x, bot.y, s.x, s.y)).map(|s| (s.x, s.y)) else { return; };
        let reachable = self.reachable(i);
        let mut best: Option<(usize, usize, usize)> = None;
        for y in 0..MAP_H { for x in 0..MAP_W {
            if !reachable[y][x] || (x == bot.x && y == bot.y) { continue; }
            let score = manhattan(x, y, goal.0, goal.1);
            if best.map(|b| score < b.0).unwrap_or(true) { best = Some((score, x, y)); }
        }}
        if let Some((_, x, y)) = best {
            self.units[i].x = x;
            self.units[i].y = y;
            self.note(format!("BOT {} advances // {},{}", kind_name(bot.kind), x + 1, y + 1));
            self.capture(x, y, BOTS, bot.kind);
        }
    }

    fn bot_damage(&mut self, ai: usize, ti: usize) {
        let attacker = self.units[ai].clone();
        let target = self.units[ti].clone();
        let cover = if self.terrain[target.y][target.x] == Terrain::Forest { 1 } else { 0 };
        let amount = (damage(attacker.kind, target.kind) - cover).max(1);
        self.units[ti].hp -= amount;
        self.note(format!("BOT FIRE // {} damage to {}", amount, kind_name(target.kind)));
        if self.units[ti].hp <= 0 { self.note("HUMAN UNIT LOST".into()); self.units.remove(ti); }
    }

    fn check_victory(&mut self) {
        let human_sites = self.sites.iter().filter(|s| s.owner == Some(HUMAN)).count();
        let bot_sites = self.sites.iter().filter(|s| s.owner == Some(BOTS)).count();
        let humans = self.units.iter().any(|u| u.team == HUMAN);
        let bots = self.units.iter().any(|u| u.team == BOTS);
        if human_sites >= 3 || (!bots && humans) { self.over = Some(HUMAN); self.note("FRONTIER SECURED // VICTORY".into()); }
        else if bot_sites >= 3 || (!humans && bots) { self.over = Some(BOTS); self.note("FRONTIER OVERRUN // DEFEAT".into()); }
    }

    fn note(&mut self, line: String) {
        self.log.push(line);
        if self.log.len() > 5 { self.log.remove(0); }
    }

    fn draw(&self) {
        clear_background(rgb(11, 17, 24));
        self.draw_header();
        self.draw_board();
        self.draw_panel();
        if let Some(winner) = self.over { self.draw_result(winner); }
    }

    fn draw_header(&self) {
        draw_rectangle(0.0, 0.0, 1280.0, 112.0, rgb(16, 25, 34));
        draw_rectangle(0.0, 108.0, 1280.0, 4.0, rgb(210, 92, 48));
        draw_text("RUST", 38.0, 56.0, 38.0, rgb(238, 241, 231));
        draw_text("// WARS", 143.0, 56.0, 38.0, rgb(211, 91, 51));
        draw_text("FIELD COMMAND  /  FRONTIER 09", 40.0, 84.0, 16.0, rgb(130, 151, 161));
        let human = self.sites.iter().filter(|s| s.owner == Some(HUMAN)).count();
        let bot = self.sites.iter().filter(|s| s.owner == Some(BOTS)).count();
        draw_text(&format!("ROUND {:02}", self.round), 755.0, 48.0, 24.0, rgb(232, 227, 208));
        draw_text(&format!("HUMAN {} / 3", human), 900.0, 48.0, 18.0, rgb(94, 200, 222));
        draw_text(&format!("BOTS {} / 3", bot), 1070.0, 48.0, 18.0, rgb(218, 103, 71));
        draw_text("CAPTURE 3 COMMAND SITES OR ELIMINATE THE ENEMY", 755.0, 78.0, 13.0, rgb(137, 151, 154));
    }

    fn draw_board(&self) {
        let unit_index = self.selected.and_then(|id| self.units.iter().position(|u| u.id == id));
        let reachable = unit_index.map(|i| self.reachable(i));
        draw_rectangle(BOARD_X - 8.0, BOARD_Y - 8.0, MAP_W as f32 * TILE + 16.0, MAP_H as f32 * TILE + 16.0, rgb(27, 38, 43));
        for y in 0..MAP_H { for x in 0..MAP_W {
            let px = BOARD_X + x as f32 * TILE;
            let py = BOARD_Y + y as f32 * TILE;
            let base = match self.terrain[y][x] { Terrain::Plain => if (x + y) % 2 == 0 { rgb(69, 91, 73) } else { rgb(63, 84, 69) }, Terrain::Forest => rgb(45, 75, 59), Terrain::Mountain => rgb(76, 81, 79) };
            draw_rectangle(px, py, TILE - 2.0, TILE - 2.0, base);
            match self.terrain[y][x] {
                Terrain::Plain => {
                    draw_line(px + 7.0, py + 54.0, px + 24.0, py + 52.0, 1.0, rgb(84, 107, 80));
                    draw_line(px + 40.0, py + 13.0, px + 55.0, py + 12.0, 1.0, rgb(84, 107, 80));
                }
                Terrain::Forest => self.draw_tree(px, py),
                Terrain::Mountain => {
                    draw_triangle(vec2(px + 11.0, py + 51.0), vec2(px + 34.0, py + 10.0), vec2(px + 57.0, py + 51.0), rgb(119, 128, 119));
                    draw_triangle(vec2(px + 25.0, py + 28.0), vec2(px + 34.0, py + 10.0), vec2(px + 43.0, py + 28.0), rgb(207, 205, 185));
                }
            }
            if let Some(site) = self.sites.iter().find(|s| s.x == x && s.y == y) { self.draw_site(px, py, site.owner); }
            if reachable.as_ref().map(|grid| grid[y][x]).unwrap_or(false) && self.unit_at(x, y).is_none() {
                draw_rectangle(px + 2.0, py + 2.0, TILE - 6.0, TILE - 6.0, Color::new(0.25, 0.8, 0.9, 0.20));
                draw_rectangle_lines(px + 4.0, py + 4.0, TILE - 10.0, TILE - 10.0, 1.0, rgb(117, 218, 222));
            }
            if let Some(u) = self.unit_at(x, y) { self.draw_unit(px, py, u); }
            if let Some(id) = self.selected {
                if self.units.iter().any(|u| u.id == id && u.x == x && u.y == y) {
                    draw_rectangle_lines(px + 2.0, py + 2.0, TILE - 6.0, TILE - 6.0, 3.0, rgb(242, 207, 107));
                }
            }
        }}
        draw_text("NORTH SECTOR // 10 × 8 GRID", BOARD_X, BOARD_Y - 18.0, 13.0, rgb(128, 149, 151));
        draw_text("FOREST: COVER + SLOW  /  MOUNTAIN: IMPASSABLE  /  INFANTRY CAPTURES SITES", BOARD_X, BOARD_Y + MAP_H as f32 * TILE + 27.0, 12.0, rgb(130, 148, 145));
    }

    fn draw_tree(&self, x: f32, y: f32) {
        draw_rectangle(x + 29.0, y + 33.0, 6.0, 18.0, rgb(116, 82, 54));
        draw_triangle(vec2(x + 12.0, y + 40.0), vec2(x + 32.0, y + 8.0), vec2(x + 52.0, y + 40.0), rgb(77, 119, 77));
        draw_triangle(vec2(x + 16.0, y + 31.0), vec2(x + 32.0, y + 4.0), vec2(x + 48.0, y + 31.0), rgb(91, 139, 85));
    }

    fn draw_site(&self, x: f32, y: f32, owner: Option<u8>) {
        let col = match owner { Some(HUMAN) => rgb(71, 183, 204), Some(BOTS) => rgb(213, 87, 60), _ => rgb(232, 189, 94) };
        draw_rectangle(x + 13.0, y + 17.0, 38.0, 32.0, rgb(26, 37, 42));
        draw_rectangle(x + 18.0, y + 24.0, 28.0, 25.0, col);
        draw_rectangle(x + 23.0, y + 18.0, 18.0, 9.0, col);
        draw_rectangle(x + 27.0, y + 11.0, 3.0, 16.0, rgb(228, 224, 205));
        draw_rectangle(x + 30.0, y + 12.0, 13.0, 7.0, col);
        draw_rectangle(x + 23.0, y + 30.0, 5.0, 7.0, rgb(27, 41, 43));
        draw_rectangle(x + 35.0, y + 30.0, 5.0, 7.0, rgb(27, 41, 43));
    }

    fn draw_unit(&self, x: f32, y: f32, u: &Unit) {
        let team = if u.team == HUMAN { rgb(67, 184, 208) } else { rgb(207, 82, 58) };
        let dark = if u.team == HUMAN { rgb(26, 101, 124) } else { rgb(104, 43, 40) };
        draw_ellipse(x + 32.0, y + 49.0, 22.0, 7.0, 0.0, Color::new(0.0, 0.0, 0.0, 0.28));
        match u.kind {
            Kind::Infantry => {
                draw_circle(x + 32.0, y + 22.0, 7.0, rgb(220, 193, 151));
                draw_rectangle(x + 24.0, y + 30.0, 16.0, 16.0, team);
                draw_rectangle(x + 21.0, y + 35.0, 5.0, 13.0, dark);
                draw_rectangle(x + 30.0, y + 45.0, 5.0, 8.0, rgb(35, 42, 42));
                draw_rectangle(x + 39.0, y + 45.0, 5.0, 8.0, rgb(35, 42, 42));
                draw_rectangle(x + 37.0, y + 31.0, 15.0, 3.0, rgb(67, 76, 74));
            }
            Kind::Tank => {
                draw_rectangle(x + 11.0, y + 37.0, 42.0, 14.0, dark);
                draw_rectangle(x + 15.0, y + 27.0, 34.0, 17.0, team);
                draw_rectangle(x + 22.0, y + 22.0, 17.0, 9.0, rgb(183, 193, 166));
                draw_rectangle(x + 36.0, y + 25.0, 22.0, 5.0, rgb(45, 57, 59));
                for wx in [18.0, 29.0, 40.0] { draw_circle(x + wx, y + 45.0, 3.0, rgb(177, 179, 159)); }
            }
            Kind::Artillery => {
                draw_rectangle(x + 12.0, y + 39.0, 39.0, 10.0, dark);
                draw_rectangle(x + 20.0, y + 31.0, 22.0, 13.0, team);
                draw_line(x + 31.0, y + 35.0, x + 48.0, y + 20.0, 5.0, rgb(191, 193, 169));
                draw_circle(x + 27.0, y + 43.0, 4.0, rgb(179, 180, 159));
                draw_circle(x + 39.0, y + 43.0, 4.0, rgb(179, 180, 159));
            }
        }
        let badge = if u.team == HUMAN { "H" } else { "B" };
        draw_rectangle(x + 4.0, y + 4.0, 18.0, 16.0, dark);
        draw_text(badge, x + 8.0, y + 17.0, 13.0, WHITE);
        draw_rectangle(x + 26.0, y + 55.0, 28.0, 5.0, rgb(29, 38, 39));
        draw_rectangle(x + 26.0, y + 55.0, 28.0 * u.hp.max(0) as f32 / 10.0, 5.0, if u.hp > 5 { rgb(124, 203, 142) } else { rgb(233, 139, 91) });
        if u.acted { draw_rectangle(x + 2.0, y + 2.0, TILE - 6.0, TILE - 6.0, Color::new(0.03, 0.05, 0.07, 0.27)); }
    }

    fn draw_panel(&self) {
        let x = 718.0;
        draw_rectangle(x, 132.0, 522.0, 548.0, rgb(20, 30, 39));
        draw_rectangle(x, 132.0, 4.0, 548.0, rgb(210, 92, 48));
        draw_text("BATTLE STATUS", x + 22.0, 165.0, 17.0, rgb(233, 224, 200));
        draw_text("YOUR SQUAD", x + 22.0, 198.0, 12.0, rgb(110, 145, 156));
        let human_units: Vec<&Unit> = self.units.iter().filter(|u| u.team == HUMAN).collect();
        for (row, u) in human_units.iter().enumerate() {
            let y = 213.0 + row as f32 * 46.0;
            draw_rectangle(x + 18.0, y, 484.0, 38.0, if self.selected == Some(u.id) { rgb(36, 60, 68) } else { rgb(27, 39, 46) });
            let col = if u.acted { rgb(115, 133, 135) } else { rgb(96, 207, 220) };
            draw_rectangle(x + 27.0, y + 8.0, 22.0, 22.0, col);
            draw_text(&format!("{}", unit_glyph(u.kind)), x + 32.0, y + 25.0, 15.0, rgb(19, 31, 37));
            draw_text(&format!("{:02}  {}", u.id, kind_name(u.kind).to_uppercase()), x + 62.0, y + 17.0, 14.0, rgb(226, 228, 211));
            draw_text(if u.acted { "SPENT" } else { "READY" }, x + 387.0, y + 16.0, 11.0, if u.acted { rgb(123, 132, 130) } else { rgb(123, 210, 153) });
            draw_text(&format!("HP {:02}", u.hp), x + 62.0, y + 32.0, 10.0, rgb(137, 158, 156));
        }
        draw_line(x + 20.0, 410.0, x + 500.0, 410.0, 1.0, rgb(53, 69, 75));
        draw_text("FIELD ORDERS", x + 22.0, 436.0, 12.0, rgb(110, 145, 156));
        let selected = self.selected.and_then(|id| self.units.iter().find(|u| u.id == id));
        if let Some(u) = selected {
            let action = if u.acted { "ACTION SPENT // END TURN TO RESET" } else { match u.kind { Kind::Infantry => "MOVE 3  /  CAPTURE COMMAND SITES", Kind::Tank => "MOVE 4  /  DIRECT FIRE: 1 TILE", Kind::Artillery => "MOVE 3  /  INDIRECT FIRE: 2–3 TILES" } };
            draw_text(&format!("SELECTED // {} #{}", kind_name(u.kind).to_uppercase(), u.id), x + 22.0, 465.0, 17.0, rgb(232, 222, 197));
            draw_text(action, x + 22.0, 488.0, 12.0, if u.acted { rgb(219, 151, 103) } else { rgb(148, 182, 163) });
        } else {
            draw_text("CLICK A UNIT TO ISSUE ORDERS", x + 22.0, 466.0, 15.0, rgb(202, 213, 201));
            draw_text("INFANTRY CAPTURES THE GOLD SITES", x + 22.0, 489.0, 12.0, rgb(148, 182, 163));
        }
        draw_text("COMMS // LAST REPORTS", x + 22.0, 532.0, 12.0, rgb(110, 145, 156));
        for (i, line) in self.log.iter().rev().take(4).enumerate() {
            draw_text(line, x + 22.0, 555.0 + i as f32 * 21.0, 12.0, if i == 0 { rgb(220, 192, 138) } else { rgb(143, 162, 160) });
        }
        draw_text("CLICK TILE: MOVE / ATTACK     ESC: DESELECT", x + 22.0, 658.0, 10.0, rgb(104, 126, 130));
        let button = Rect::new(1020.0, 710.0, 212.0, 48.0);
        let hovered = button.contains(vec2(mouse_position().0, mouse_position().1));
        draw_rectangle(button.x, button.y, button.w, button.h, if hovered { rgb(223, 111, 66) } else { rgb(195, 86, 53) });
        draw_text("END TURN  [ E ]", button.x + 31.0, button.y + 31.0, 18.0, rgb(252, 239, 217));
        draw_text("DEMO BUILD  //  PROTOTYPE", 38.0, 756.0, 11.0, rgb(87, 111, 113));
    }

    fn draw_result(&self, winner: u8) {
        draw_rectangle(0.0, 0.0, 1280.0, 800.0, Color::new(0.02, 0.04, 0.06, 0.82));
        let title = if winner == HUMAN { "FRONTIER SECURED" } else { "FRONTIER OVERRUN" };
        let subtitle = if winner == HUMAN { "HUMANITY HOLDS THE LINE" } else { "THE WAR BOTS TAKE THE SECTOR" };
        draw_text(title, 330.0, 356.0, 48.0, if winner == HUMAN { rgb(103, 217, 221) } else { rgb(231, 113, 79) });
        draw_text(subtitle, 451.0, 399.0, 18.0, rgb(226, 221, 197));
        draw_text("PRESS ENTER OR CLICK TO DEPLOY AGAIN", 435.0, 449.0, 15.0, rgb(143, 165, 164));
    }

    fn unit_at(&self, x: usize, y: usize) -> Option<&Unit> { self.units.iter().find(|u| u.x == x && u.y == y) }
}

fn can_attack(kind: Kind, ax: usize, ay: usize, tx: usize, ty: usize) -> bool {
    let d = manhattan(ax, ay, tx, ty);
    match kind { Kind::Artillery => (2..=3).contains(&d), _ => d == 1 }
}

fn damage(attacker: Kind, defender: Kind) -> i32 {
    match (attacker, defender) {
        (Kind::Tank, Kind::Infantry) => 7,
        (Kind::Tank, _) => 5,
        (Kind::Infantry, Kind::Tank) => 2,
        (Kind::Infantry, _) => 4,
        (Kind::Artillery, Kind::Tank) => 6,
        (Kind::Artillery, _) => 5,
    }
}

fn move_range(kind: Kind) -> i32 { if kind == Kind::Tank { 4 } else { 3 } }
fn kind_name(kind: Kind) -> &'static str { match kind { Kind::Infantry => "Infantry", Kind::Tank => "Armor", Kind::Artillery => "Artillery" } }
fn unit_glyph(kind: Kind) -> &'static str { match kind { Kind::Infantry => "I", Kind::Tank => "T", Kind::Artillery => "A" } }
fn manhattan(x: usize, y: usize, gx: usize, gy: usize) -> usize { x.abs_diff(gx) + y.abs_diff(gy) }
fn neighbors(x: usize, y: usize) -> [(usize, usize); 4] { [(x.saturating_sub(1), y), (x + 1, y), (x, y.saturating_sub(1)), (x, y + 1)] }
fn rgb(r: u8, g: u8, b: u8) -> Color { Color::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0) }

fn window_conf() -> Conf {
    Conf { window_title: "RUST // WARS — Frontier 09".to_string(), window_width: 1280, window_height: 800, window_resizable: false, ..Default::default() }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = Game::new();
    loop {
        game.update();
        game.draw();
        next_frame().await;
    }
}
