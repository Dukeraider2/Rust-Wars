use macroquad::{audio, audio::{PlaySoundParams, Sound}, prelude::*};
use std::f32::consts::PI;

const MAP_W: usize = 16;
const MAP_H: usize = 12;
const ISO_TILE_W: f32 = 64.0;
const ISO_TILE_H: f32 = 32.0;
const ELEVATION_STEP: f32 = 9.0;
const VIEW_CENTER_X: f32 = 489.0;
const VIEW_CENTER_Y: f32 = 420.0;
const VIEW_LEFT: f32 = 20.0;
const VIEW_RIGHT: f32 = 958.0;
const VIEW_TOP: f32 = 154.0;
const VIEW_BOTTOM: f32 = 664.0;
const HUMAN: u8 = 0;
const BOTS: u8 = 1;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Terrain { Plain, Forest, Mountain }

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind { Infantry, Tank, Artillery }

#[derive(Clone, Copy, PartialEq, Eq)]
enum Sfx { Select, Move, Fire, Impact, Capture }

impl Sfx {
    const ALL: [Sfx; 5] = [Sfx::Select, Sfx::Move, Sfx::Fire, Sfx::Impact, Sfx::Capture];
    fn index(self) -> usize { match self { Sfx::Select => 0, Sfx::Move => 1, Sfx::Fire => 2, Sfx::Impact => 3, Sfx::Capture => 4 } }
}

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
    elevation: [[u8; MAP_W]; MAP_H],
    units: Vec<Unit>,
    sites: Vec<Site>,
    selected: Option<usize>,
    target: Option<usize>,
    round: usize,
    over: Option<u8>,
    log: Vec<String>,
    music_on: bool,
    camera: Vec2,
    last_mouse: Vec2,
    dragging: bool,
    drag_with_left: bool,
    pan_mode: bool,
    sfx_queue: Vec<Sfx>,
}

impl Game {
    fn new() -> Self {
        let mut terrain = [[Terrain::Plain; MAP_W]; MAP_H];
        for (x, y) in [(2,2),(3,2),(7,1),(8,1),(10,3),(12,4),(6,10),(11,8),(13,10),(3,6),(5,8),(9,4),(14,6),(5,5)] {
            terrain[y][x] = Terrain::Forest;
        }
        for (x, y) in [(5,3),(6,3),(5,4),(9,7),(10,7),(11,7),(3,9),(14,4)] { terrain[y][x] = Terrain::Mountain; }
        let mut elevation = [[0_u8; MAP_W]; MAP_H];
        for (x, y) in [(4,3),(5,3),(6,3),(4,4),(5,4),(6,4),(7,4),(8,4),(5,5),(6,5),(7,5),(10,6),(11,6),(12,6),(11,7),(12,7),(3,8),(4,8),(4,9),(13,5),(14,6),(13,7),(2,7),(1,9),(2,10)] { elevation[y][x] = 1; }
        for (x, y) in [(5,3),(6,3),(5,4),(9,7),(10,7),(11,7),(3,9),(14,4)] { elevation[y][x] = 3; }
        for (x, y) in [(8,3),(7,3),(12,5),(13,6),(2,8),(2,9)] { elevation[y][x] = 2; }
        let units = vec![
            Unit { id: 1, team: HUMAN, kind: Kind::Infantry, x: 1, y: 10, hp: 10, acted: false },
            Unit { id: 2, team: HUMAN, kind: Kind::Tank, x: 2, y: 11, hp: 10, acted: false },
            Unit { id: 3, team: HUMAN, kind: Kind::Artillery, x: 0, y: 11, hp: 10, acted: false },
            Unit { id: 4, team: HUMAN, kind: Kind::Infantry, x: 3, y: 10, hp: 10, acted: false },
            Unit { id: 5, team: HUMAN, kind: Kind::Tank, x: 4, y: 11, hp: 10, acted: false },
            Unit { id: 6, team: BOTS, kind: Kind::Infantry, x: 14, y: 1, hp: 10, acted: false },
            Unit { id: 7, team: BOTS, kind: Kind::Tank, x: 15, y: 0, hp: 10, acted: false },
            Unit { id: 8, team: BOTS, kind: Kind::Artillery, x: 15, y: 2, hp: 10, acted: false },
            Unit { id: 9, team: BOTS, kind: Kind::Tank, x: 13, y: 0, hp: 10, acted: false },
            Unit { id: 10, team: BOTS, kind: Kind::Infantry, x: 12, y: 1, hp: 10, acted: false },
        ];
        let sites = vec![
            Site { x: 3, y: 8, owner: None }, Site { x: 6, y: 6, owner: None },
            Site { x: 8, y: 4, owner: None }, Site { x: 11, y: 5, owner: None },
            Site { x: 13, y: 8, owner: None },
        ];
        Self {
            terrain, elevation, units, sites, selected: None, target: None, round: 1, over: None,
            log: vec!["FRONTIER 09 // capture three command sites".to_string(), "Select a unit, then click a highlighted tile.".to_string()],
            music_on: true,
            camera: vec2(0.0, -16.0), last_mouse: vec2(0.0, 0.0), dragging: false,
            drag_with_left: false, pan_mode: false,
            sfx_queue: Vec::new(),
        }
    }

    fn update(&mut self) {
        if is_key_pressed(KeyCode::M) { self.music_on = !self.music_on; }
        if self.over.is_some() {
            if is_key_pressed(KeyCode::Enter) || is_mouse_button_pressed(MouseButton::Left) { *self = Self::new(); }
            return;
        }
        if is_key_pressed(KeyCode::Tab) { self.cycle_target(); }
        if is_key_pressed(KeyCode::Escape) { self.selected = None; }
        if is_key_pressed(KeyCode::E) { self.end_turn(); }
        if is_key_pressed(KeyCode::P) { self.pan_mode = !self.pan_mode; self.dragging = false; }
        if is_key_pressed(KeyCode::Home) { self.camera = vec2(0.0, -16.0); }
        let pan_speed = 360.0 * get_frame_time();
        if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) { self.camera.x -= pan_speed; }
        if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) { self.camera.x += pan_speed; }
        if is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) { self.camera.y -= pan_speed; }
        if is_key_down(KeyCode::Down) || is_key_down(KeyCode::S) { self.camera.y += pan_speed; }
        let (mx, my) = mouse_position();
        let mouse = vec2(mx, my);
        if is_mouse_button_pressed(MouseButton::Middle) && in_map_view(mx, my) {
            self.dragging = true;
            self.drag_with_left = false;
            self.last_mouse = mouse;
        }
        if self.pan_mode && is_mouse_button_pressed(MouseButton::Left) && in_map_view(mx, my) {
            self.dragging = true;
            self.drag_with_left = true;
            self.last_mouse = mouse;
        }
        let pan_button_down = if self.drag_with_left { is_mouse_button_down(MouseButton::Left) } else { is_mouse_button_down(MouseButton::Middle) };
        if self.dragging && pan_button_down {
            let delta = mouse - self.last_mouse;
            self.camera -= delta;
            self.last_mouse = mouse;
        }
        if (self.drag_with_left && is_mouse_button_released(MouseButton::Left)) || (!self.drag_with_left && is_mouse_button_released(MouseButton::Middle)) { self.dragging = false; }
        self.camera.x = self.camera.x.clamp(-778.0, 788.0);
        self.camera.y = self.camera.y.clamp(-454.0, 494.0);
        if is_mouse_button_pressed(MouseButton::Left) {
            if Rect::new(1020.0, 710.0, 212.0, 48.0).contains(vec2(mx, my)) { self.end_turn(); }
            else if !self.pan_mode {
                if let Some((x, y)) = tile_from_screen(mx, my, self.camera, &self.elevation) { self.click_tile(x, y); }
                else if mx < VIEW_RIGHT { self.selected = None; }
            }
        }
        if is_mouse_button_pressed(MouseButton::Right) { self.selected = None; }
    }

    fn cycle_target(&mut self) {
        let enemies: Vec<usize> = self.units.iter().filter(|u| u.team == BOTS).map(|u| u.id).collect();
        if enemies.is_empty() { self.target = None; return; }
        let next = self.target.and_then(|id| enemies.iter().position(|candidate| *candidate == id).map(|i| (i + 1) % enemies.len())).unwrap_or(0);
        self.target = Some(enemies[next]);
        if let Some(unit) = self.units.iter().find(|u| Some(u.id) == self.target).cloned() {
            self.camera = tile_world(unit.x, unit.y);
            self.sfx_queue.push(Sfx::Select);
            self.note(format!("TARGET LOCK // BOT {} #{} // {} HP", kind_name(unit.kind).to_uppercase(), unit.id, unit.hp));
        }
    }

    fn click_tile(&mut self, x: usize, y: usize) {
        if let Some(i) = self.units.iter().position(|u| u.x == x && u.y == y) {
            let unit = self.units[i].clone();
            if unit.team == HUMAN {
                self.selected = Some(unit.id);
                self.sfx_queue.push(Sfx::Select);
                self.note(format!("{} selected // {} HP", kind_name(unit.kind), unit.hp));
            } else {
                self.target = Some(unit.id);
                let can_fire = self.selected.and_then(|id| self.units.iter().find(|u| u.id == id))
                    .map(|attacker| !attacker.acted && self.can_attack(attacker.kind, attacker.x, attacker.y, unit.x, unit.y)).unwrap_or(false);
                if can_fire { if let Some(id) = self.selected { self.attack(id, unit.id); } }
                else { self.sfx_queue.push(Sfx::Select); self.note(format!("TARGET LOCK // BOT {} #{} // TAB CYCLES ENEMIES", kind_name(unit.kind).to_uppercase(), unit.id)); }
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
        self.sfx_queue.push(Sfx::Move);
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
        if !self.can_attack(a.kind, a.x, a.y, t.x, t.y) {
            self.note(if a.kind == Kind::Artillery && in_weapon_range(a.kind, a.x, a.y, t.x, t.y) { "No clear artillery lane to target." } else { "Target is outside weapon range." }.into());
            return;
        }
        self.sfx_queue.push(Sfx::Fire);
        self.deal_damage(ai, ti);
        self.units[ai].acted = true;
        if !self.units.iter().any(|u| u.id == target_id) { self.target = None; }
        self.check_victory();
    }

    fn deal_damage(&mut self, ai: usize, ti: usize) {
        let attacker = self.units[ai].clone();
        let target = self.units[ti].clone();
        let amount = self.attack_damage(&attacker, &target);
        self.units[ti].hp -= amount;
        self.sfx_queue.push(Sfx::Impact);
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
        if captured { self.sfx_queue.push(Sfx::Capture); self.note(if team == HUMAN { "COMMAND SITE SECURED // HUMAN CONTROL".into() } else { "COMMAND SITE LOST // BOT CONTROL".into() }); }
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
                    let climb = self.elevation[ny][nx].saturating_sub(self.elevation[y][x]);
                    if climb > 1 { continue; }
                    let terrain_cost = if self.terrain[ny][nx] == Terrain::Forest { 2 } else { 1 };
                    let cost = terrain_cost + climb as i32;
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

    fn can_attack(&self, kind: Kind, ax: usize, ay: usize, tx: usize, ty: usize) -> bool {
        if !in_weapon_range(kind, ax, ay, tx, ty) { return false; }
        let distance = manhattan(ax, ay, tx, ty);
        // Artillery fires along a clear cardinal lane; intervening ridges can mask low targets.
        if kind == Kind::Artillery {
            if ax != tx && ay != ty { return false; }
            for step in 1..distance {
                let (x, y) = if ax == tx {
                    (ax, if ty > ay { ay + step } else { ay - step })
                } else {
                    (if tx > ax { ax + step } else { ax - step }, ay)
                };
                let sight_height = self.elevation[ay][ax] as f32
                    + (self.elevation[ty][tx] as f32 - self.elevation[ay][ax] as f32) * step as f32 / distance as f32;
                if self.terrain[y][x] == Terrain::Mountain || self.elevation[y][x] as f32 > sight_height { return false; }
            }
        }
        true
    }

    fn attack_damage(&self, attacker: &Unit, target: &Unit) -> i32 {
        let attack_height = self.elevation[attacker.y][attacker.x];
        let target_height = self.elevation[target.y][target.x];
        let high_ground_bonus = i32::from(attack_height > target_height);
        let height_cover = i32::from(target_height > attack_height);
        let forest_cover = i32::from(self.terrain[target.y][target.x] == Terrain::Forest);
        (damage(attacker.kind, target.kind) + high_ground_bonus - height_cover - forest_cover).max(1)
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
        if let Some(t) = self.units.iter().find(|u| u.team == HUMAN && self.can_attack(bot.kind, bot.x, bot.y, u.x, u.y)).cloned() {
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
            self.sfx_queue.push(Sfx::Move);
            self.note(format!("BOT {} advances // {},{}", kind_name(bot.kind), x + 1, y + 1));
            self.capture(x, y, BOTS, bot.kind);
        }
    }

    fn bot_damage(&mut self, ai: usize, ti: usize) {
        let attacker = self.units[ai].clone();
        let target = self.units[ti].clone();
        let amount = self.attack_damage(&attacker, &target);
        self.units[ti].hp -= amount;
        self.sfx_queue.push(Sfx::Fire);
        self.sfx_queue.push(Sfx::Impact);
        self.note(format!("BOT FIRE // {} damage to {}", amount, kind_name(target.kind)));
        if self.units[ti].hp <= 0 { self.note("HUMAN UNIT LOST".into()); self.units.remove(ti); }
    }

    fn check_victory(&mut self) {
        let human_sites = self.sites.iter().filter(|s| s.owner == Some(HUMAN)).count();
        let bot_sites = self.sites.iter().filter(|s| s.owner == Some(BOTS)).count();
        let humans = self.units.iter().any(|u| u.team == HUMAN);
        let bots = self.units.iter().any(|u| u.team == BOTS);
        if human_sites >= 3 || (!bots && humans) { self.over = Some(HUMAN); self.sfx_queue.push(Sfx::Capture); self.note("FRONTIER SECURED // VICTORY".into()); }
        else if bot_sites >= 3 || (!humans && bots) { self.over = Some(BOTS); self.sfx_queue.push(Sfx::Impact); self.note("FRONTIER OVERRUN // DEFEAT".into()); }
    }

    fn note(&mut self, line: String) {
        self.log.push(line);
        if self.log.len() > 5 { self.log.remove(0); }
    }

    fn draw(&self, unit_atlas: &Texture2D) {
        clear_background(rgb(11, 17, 24));
        self.draw_header();
        self.draw_board(unit_atlas);
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
        draw_text(if self.music_on { "MUSIC ON  [M]" } else { "MUSIC OFF [M]" }, 1110.0, 25.0, 12.0, if self.music_on { rgb(123, 210, 153) } else { rgb(145, 151, 148) });
        draw_rectangle(VIEW_LEFT, 118.0, VIEW_RIGHT - VIEW_LEFT, 30.0, rgb(23, 36, 43));
        let order = self.selected.and_then(|id| self.units.iter().find(|u| u.id == id))
            .map(|u| format!("ORDER {} #{}  HP {}  ELEV {}", kind_name(u.kind).to_uppercase(), u.id, u.hp, self.elevation[u.y][u.x]))
            .unwrap_or_else(|| "ORDER: SELECT A HUMAN UNIT".to_string());
        let target = self.target.and_then(|id| self.units.iter().find(|u| u.id == id))
            .map(|u| format!("TARGET BOT {} #{}  HP {}  ELEV {}", kind_name(u.kind).to_uppercase(), u.id, u.hp, self.elevation[u.y][u.x]))
            .unwrap_or_else(|| format!("{} BOTS  //  TAB CYCLES TARGET", self.units.iter().filter(|u| u.team == BOTS).count()));
        draw_text(&format!("{}     |     {}", order, target), VIEW_LEFT + 12.0, 138.0, 12.0, rgb(205, 216, 203));
    }

    fn draw_board(&self, unit_atlas: &Texture2D) {
        let unit_index = self.selected.and_then(|id| self.units.iter().position(|u| u.id == id));
        let reachable = unit_index.map(|i| self.reachable(i));
        draw_rectangle(VIEW_LEFT, VIEW_TOP, VIEW_RIGHT - VIEW_LEFT, VIEW_BOTTOM - VIEW_TOP, rgb(17, 28, 34));
        draw_text("NORTH SECTOR // 16 Ã— 12 ISO GRID", 38.0, 174.0, 13.0, rgb(128, 149, 151));
        for sum in 0..MAP_W + MAP_H - 1 {
            for y in 0..MAP_H { for x in 0..MAP_W {
                if x + y != sum { continue; }
                let (cx, cy) = tile_center(x, y, self.camera, self.elevation[y][x]);
                if cx < VIEW_LEFT - 32.0 || cx > VIEW_RIGHT + 32.0 || cy < VIEW_TOP + 48.0 || cy > VIEW_BOTTOM - 12.0 { continue; }
                self.draw_elevation_faces(x, y, cx, cy);
                let base = match self.terrain[y][x] {
                    Terrain::Plain => if (x + y) % 2 == 0 { rgb(76, 88, 65) } else { rgb(67, 79, 59) },
                    Terrain::Forest => rgb(48, 75, 55),
                    Terrain::Mountain => rgb(77, 84, 80),
                };
                draw_diamond(cx, cy, ISO_TILE_W * 0.5, ISO_TILE_H * 0.5, base);
                // Panel seams and scattered flecks give the ground a built-up RTS battlefield feel.
                draw_line(cx - 15.0, cy - 7.0, cx, cy, 1.0, Color::new(0.76, 0.78, 0.66, 0.13));
                if (x * 7 + y * 11) % 5 == 0 {
                    draw_circle(cx + ((x * 13 + y * 3) % 19) as f32 - 9.0, cy + ((x * 5 + y * 7) % 7) as f32 - 3.0, 1.3, Color::new(0.79, 0.74, 0.57, 0.32));
                }
                draw_diamond_outline(cx, cy, ISO_TILE_W * 0.5, ISO_TILE_H * 0.5, 1.0, rgb(38, 52, 45));
                if self.elevation[y][x] > 0 && self.terrain[y][x] != Terrain::Mountain {
                    draw_diamond_outline(cx, cy, 19.0, 9.5, 1.0, Color::new(0.91, 0.78, 0.46, 0.45));
                    for level in 0..self.elevation[y][x] { draw_line(cx - 4.0 + level as f32 * 4.0, cy + 5.0, cx - 1.0 + level as f32 * 4.0, cy + 2.0, 1.0, rgb(226, 198, 126)); }
                }
                if reachable.as_ref().map(|grid| grid[y][x]).unwrap_or(false) && self.unit_at(x, y).is_none() {
                    draw_diamond(cx, cy, ISO_TILE_W * 0.46, ISO_TILE_H * 0.44, Color::new(0.25, 0.8, 0.9, 0.22));
                    draw_diamond_outline(cx, cy, ISO_TILE_W * 0.44, ISO_TILE_H * 0.42, 1.5, rgb(117, 218, 222));
                }
                if let Some(site) = self.sites.iter().find(|s| s.x == x && s.y == y) { self.draw_site(cx, cy, site.owner); }
                match self.terrain[y][x] {
                    Terrain::Plain => {
                        draw_line(cx - 19.0, cy + 1.0, cx - 9.0, cy + 3.0, 1.0, rgb(91, 119, 83));
                        draw_line(cx + 11.0, cy - 5.0, cx + 20.0, cy - 3.0, 1.0, rgb(91, 119, 83));
                    }
                    Terrain::Forest => self.draw_tree(cx, cy),
                    Terrain::Mountain => self.draw_mountain(cx, cy),
                }
                if let Some(u) = self.unit_at(x, y) { self.draw_unit(cx, cy, u, unit_atlas); }
                if self.selected.and_then(|id| self.units.iter().find(|u| u.id == id)).is_some_and(|u| u.x == x && u.y == y) {
                    let pulse = 2.0 + (get_time() as f32 * 4.0).sin().abs() * 1.5;
                    draw_diamond_outline(cx, cy, ISO_TILE_W * 0.47, ISO_TILE_H * 0.46, pulse, rgb(242, 207, 107));
                }
                if self.target.and_then(|id| self.units.iter().find(|u| u.id == id)).is_some_and(|u| u.x == x && u.y == y) {
                    draw_diamond_outline(cx, cy, ISO_TILE_W * 0.49, ISO_TILE_H * 0.49, 2.5, rgb(242, 119, 81));
                }
            }}
        }
        draw_text(if self.pan_mode { "PAN MODE ON // DRAG MAP   /   P TO EXIT   /   HOME RESET" } else { "P: PAN MODE + DRAG   /   MIDDLE-DRAG   /   ARROWS PAN   /   HOME RESET" }, 38.0, 692.0, 10.0, rgb(130, 148, 145));
        draw_text("FOREST: COVER + SLOW  /  HIGH GROUND: +DMG +DEF  /  CLIMB: +MOVE  /  RIDGES BLOCK FIRE", 38.0, 675.0, 10.0, rgb(104, 126, 130));
    }

    fn draw_elevation_faces(&self, x: usize, y: usize, cx: f32, cy: f32) {
        let height = self.elevation[y][x];
        if height == 0 { return; }
        let right = vec2(cx + ISO_TILE_W * 0.5, cy);
        let bottom = vec2(cx, cy + ISO_TILE_H * 0.5);
        let left = vec2(cx - ISO_TILE_W * 0.5, cy);
        if x + 1 < MAP_W {
            let drop = height.saturating_sub(self.elevation[y][x + 1]) as f32 * ELEVATION_STEP;
            if drop > 0.0 { draw_cliff_face(right, bottom, drop, rgb(51, 57, 48), rgb(112, 95, 66)); }
        }
        if y + 1 < MAP_H {
            let drop = height.saturating_sub(self.elevation[y + 1][x]) as f32 * ELEVATION_STEP;
            if drop > 0.0 { draw_cliff_face(bottom, left, drop, rgb(42, 49, 44), rgb(84, 83, 62)); }
        }
    }

    fn draw_tree(&self, cx: f32, cy: f32) {
        draw_diamond(cx, cy + 2.0, 13.0, 6.0, rgb(39, 62, 48));
        draw_rectangle(cx - 2.0, cy - 17.0, 4.0, 20.0, rgb(116, 82, 54));
        draw_triangle(vec2(cx - 17.0, cy - 12.0), vec2(cx, cy - 44.0), vec2(cx + 17.0, cy - 12.0), rgb(62, 105, 67));
        draw_triangle(vec2(cx - 14.0, cy - 22.0), vec2(cx, cy - 49.0), vec2(cx + 14.0, cy - 22.0), rgb(91, 139, 85));
        draw_triangle(vec2(cx - 8.0, cy - 32.0), vec2(cx, cy - 51.0), vec2(cx + 8.0, cy - 32.0), rgb(124, 164, 100));
    }

    fn draw_mountain(&self, cx: f32, cy: f32) {
        let left = vec2(cx - 22.0, cy);
        let back = vec2(cx, cy - 12.0);
        let right = vec2(cx + 22.0, cy);
        let front = vec2(cx, cy + 12.0);
        let peak = vec2(cx + 1.0, cy - 37.0);
        draw_triangle(peak, left, front, rgb(91, 101, 98));
        draw_triangle(peak, front, right, rgb(126, 135, 124));
        draw_triangle(peak, back, right, rgb(160, 163, 145));
        draw_triangle(vec2(cx - 5.0, cy - 27.0), vec2(cx + 1.0, cy - 37.0), vec2(cx + 8.0, cy - 25.0), rgb(222, 220, 193));
    }

    fn draw_site(&self, cx: f32, cy: f32, owner: Option<u8>) {
        let col = match owner { Some(HUMAN) => rgb(71, 183, 204), Some(BOTS) => rgb(213, 87, 60), _ => rgb(232, 189, 94) };
        draw_diamond(cx, cy + 2.0, 20.0, 9.0, rgb(35, 47, 48));
        draw_rectangle(cx - 13.0, cy - 15.0, 26.0, 18.0, col);
        draw_triangle(vec2(cx - 16.0, cy - 15.0), vec2(cx, cy - 25.0), vec2(cx + 16.0, cy - 15.0), rgb(211, 201, 168));
        draw_rectangle(cx - 4.0, cy - 8.0, 8.0, 11.0, rgb(32, 45, 48));
        draw_rectangle(cx + 9.0, cy - 30.0, 2.0, 20.0, rgb(229, 224, 204));
        draw_triangle(vec2(cx + 11.0, cy - 29.0), vec2(cx + 24.0, cy - 26.0), vec2(cx + 11.0, cy - 20.0), col);
    }

    fn draw_unit(&self, cx: f32, cy: f32, u: &Unit, unit_atlas: &Texture2D) {
        let dark = if u.team == HUMAN { rgb(26, 101, 124) } else { rgb(104, 43, 40) };
        draw_ellipse(cx, cy + 2.0, 29.0, 12.0, 0.0, Color::new(0.0, 0.0, 0.0, 0.28));
        let col = match u.kind { Kind::Infantry => 0.0, Kind::Tank => 64.0, Kind::Artillery => 128.0 };
        let row = if u.team == HUMAN { 0.0 } else { 64.0 };
        draw_texture_ex(unit_atlas, cx - 40.0, cy - 75.0, WHITE, DrawTextureParams {
            dest_size: Some(vec2(80.0, 80.0)),
            source: Some(Rect::new(col, row, 64.0, 64.0)),
            ..Default::default()
        });
        let badge = if u.team == HUMAN { "H" } else { "B" };
        draw_rectangle(cx - 25.0, cy - 43.0, 19.0, 15.0, dark);
        draw_text(badge, cx - 21.0, cy - 31.0, 12.0, WHITE);
        draw_text(&format!("{}", u.id), cx - 3.0, cy - 31.0, 11.0, rgb(232, 227, 208));
        draw_rectangle(cx - 14.0, cy + 8.0, 28.0, 4.0, rgb(29, 38, 39));
        draw_rectangle(cx - 14.0, cy + 8.0, 28.0 * u.hp.max(0) as f32 / 10.0, 4.0, if u.hp > 5 { rgb(124, 203, 142) } else { rgb(233, 139, 91) });
        if u.acted { draw_diamond(cx, cy, ISO_TILE_W * 0.46, ISO_TILE_H * 0.42, Color::new(0.03, 0.05, 0.07, 0.32)); }
    }

    fn draw_panel(&self) {
        let x = 978.0;
        draw_rectangle(x, 132.0, 282.0, 548.0, rgb(20, 30, 39));
        draw_rectangle(x, 132.0, 4.0, 548.0, rgb(210, 92, 48));
        draw_text("BATTLE STATUS", x + 16.0, 165.0, 15.0, rgb(233, 224, 200));
        draw_text("SQUAD // FIVE UNITS", x + 16.0, 198.0, 10.0, rgb(110, 145, 156));
        let human_units: Vec<&Unit> = self.units.iter().filter(|u| u.team == HUMAN).collect();
        for (row, u) in human_units.iter().enumerate() {
            let y = 205.0 + row as f32 * 34.0;
            draw_rectangle(x + 14.0, y, 254.0, 30.0, if self.selected == Some(u.id) { rgb(36, 60, 68) } else { rgb(27, 39, 46) });
            let col = if u.acted { rgb(115, 133, 135) } else { rgb(96, 207, 220) };
            draw_rectangle(x + 21.0, y + 5.0, 20.0, 20.0, col);
            draw_text(unit_glyph(u.kind), x + 27.0, y + 20.0, 13.0, rgb(19, 31, 37));
            draw_text(&format!("{:02} {}", u.id, kind_name(u.kind).to_uppercase()), x + 47.0, y + 19.0, 9.0, rgb(226, 228, 211));
            draw_text(&format!("{}hp", u.hp), x + 158.0, y + 19.0, 9.0, rgb(137, 158, 156));
            draw_text(if u.acted { "SPENT" } else { "READY" }, x + 196.0, y + 19.0, 8.0, if u.acted { rgb(123, 132, 130) } else { rgb(123, 210, 153) });
        }
        draw_line(x + 16.0, 384.0, x + 266.0, 384.0, 1.0, rgb(53, 69, 75));
        draw_text("FIELD ORDERS", x + 16.0, 407.0, 10.0, rgb(110, 145, 156));
        let selected = self.selected.and_then(|id| self.units.iter().find(|u| u.id == id));
        if let Some(u) = selected {
            let action = if u.acted { "ACTION SPENT // END TURN TO RESET" } else { match u.kind { Kind::Infantry => "MOVE 3 / CAPTURE SITES", Kind::Tank => "MOVE 4 / DIRECT FIRE: 1 TILE", Kind::Artillery => "MOVE 3 / INDIRECT FIRE: 2-3 TILES" } };
            draw_text(&format!("{} #{}  HP {}  Z{}", kind_name(u.kind).to_uppercase(), u.id, u.hp, self.elevation[u.y][u.x]), x + 16.0, 434.0, 12.0, rgb(232, 222, 197));
            draw_text(action, x + 16.0, 455.0, 8.0, if u.acted { rgb(219, 151, 103) } else { rgb(148, 182, 163) });
        } else {
            draw_text("SELECT A UNIT FOR ORDERS", x + 16.0, 435.0, 10.0, rgb(202, 213, 201));
            draw_text("INFANTRY CAPTURES SITES", x + 16.0, 455.0, 9.0, rgb(148, 182, 163));
        }
        if let Some(id) = self.target {
            if let Some(enemy) = self.units.iter().find(|u| u.id == id) {
                draw_text(&format!("TARGET // BOT {} #{}  HP {} Z{}", kind_name(enemy.kind).to_uppercase(), enemy.id, enemy.hp, self.elevation[enemy.y][enemy.x]), x + 16.0, 480.0, 9.0, rgb(240, 139, 103));
            }
        } else {
            draw_text("TARGET // CLICK BOT OR PRESS TAB", x + 16.0, 480.0, 9.0, rgb(143, 162, 160));
        }
        draw_text("COMMS // REPORTS", x + 16.0, 520.0, 10.0, rgb(110, 145, 156));
        for (i, line) in self.log.iter().rev().take(4).enumerate() {
            let compact: String = line.chars().take(39).collect();
            draw_text(&compact, x + 16.0, 541.0 + i as f32 * 20.0, 8.0, if i == 0 { rgb(220, 192, 138) } else { rgb(143, 162, 160) });
        }
        draw_text("CLICK: ORDER / TAB: TARGET / M: MUSIC", x + 16.0, 658.0, 8.0, rgb(104, 126, 130));
        let button = Rect::new(1020.0, 710.0, 212.0, 48.0);
        let hovered = button.contains(vec2(mouse_position().0, mouse_position().1));
        draw_rectangle(button.x, button.y, button.w, button.h, if hovered { rgb(223, 111, 66) } else { rgb(195, 86, 53) });
        draw_text("END TURN  [ E ]", button.x + 31.0, button.y + 31.0, 18.0, rgb(252, 239, 217));
        draw_text("DEMO BUILD // PROTOTYPE", 38.0, 756.0, 11.0, rgb(87, 111, 113));
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

fn in_weapon_range(kind: Kind, ax: usize, ay: usize, tx: usize, ty: usize) -> bool {
    let d = manhattan(ax, ay, tx, ty);
    match kind { Kind::Artillery => (2..=3).contains(&d), _ => d == 1 }
}

fn tile_world(x: usize, y: usize) -> Vec2 {
    let diagonal_x = x as f32 - y as f32 - (MAP_W as f32 - MAP_H as f32) * 0.5;
    let diagonal_y = x as f32 + y as f32 - (MAP_W as f32 + MAP_H as f32 - 2.0) * 0.5;
    vec2(diagonal_x * ISO_TILE_W * 0.5, diagonal_y * ISO_TILE_H * 0.5)
}

fn tile_center(x: usize, y: usize, camera: Vec2, elevation: u8) -> (f32, f32) {
    let world = tile_world(x, y) - camera;
    (VIEW_CENTER_X + world.x, VIEW_CENTER_Y + world.y - elevation as f32 * ELEVATION_STEP)
}

fn tile_from_screen(px: f32, py: f32, camera: Vec2, elevation: &[[u8; MAP_W]; MAP_H]) -> Option<(usize, usize)> {
    if !in_map_view(px, py) { return None; }
    let mut closest: Option<(f32, usize, usize)> = None;
    for y in 0..MAP_H { for x in 0..MAP_W {
        let (cx, cy) = tile_center(x, y, camera, elevation[y][x]);
        let score = (px - cx).abs() / (ISO_TILE_W * 0.5) + (py - cy).abs() / (ISO_TILE_H * 0.5);
        let top_priority = score - elevation[y][x] as f32 * 0.001;
        if score <= 1.0 && closest.map(|best| top_priority < best.0).unwrap_or(true) {
            closest = Some((top_priority, x, y));
        }
    }}
    closest.map(|(_, x, y)| (x, y))
}

fn in_map_view(x: f32, y: f32) -> bool { x >= VIEW_LEFT && x <= VIEW_RIGHT && y >= VIEW_TOP && y <= VIEW_BOTTOM }

fn draw_diamond(cx: f32, cy: f32, rx: f32, ry: f32, color: Color) {
    let top = vec2(cx, cy - ry);
    let right = vec2(cx + rx, cy);
    let bottom = vec2(cx, cy + ry);
    let left = vec2(cx - rx, cy);
    draw_triangle(top, right, bottom, color);
    draw_triangle(top, bottom, left, color);
}

fn draw_diamond_outline(cx: f32, cy: f32, rx: f32, ry: f32, thickness: f32, color: Color) {
    let top = vec2(cx, cy - ry);
    let right = vec2(cx + rx, cy);
    let bottom = vec2(cx, cy + ry);
    let left = vec2(cx - rx, cy);
    draw_line(top.x, top.y, right.x, right.y, thickness, color);
    draw_line(right.x, right.y, bottom.x, bottom.y, thickness, color);
    draw_line(bottom.x, bottom.y, left.x, left.y, thickness, color);
    draw_line(left.x, left.y, top.x, top.y, thickness, color);
}

fn draw_cliff_face(a: Vec2, b: Vec2, drop: f32, side: Color, edge: Color) {
    let lower_a = a + vec2(0.0, drop);
    let lower_b = b + vec2(0.0, drop);
    draw_triangle(a, b, lower_b, side);
    draw_triangle(a, lower_b, lower_a, side);
    draw_line(lower_a.x, lower_a.y, lower_b.x, lower_b.y, 1.0, edge);
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

fn decode_base64(encoded: &str) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(encoded.len() * 3 / 4);
    let mut accumulator = 0_u32;
    let mut bits = 0_u8;
    for ch in encoded.bytes() {
        let value = match ch {
            b'A'..=b'Z' => ch - b'A', b'a'..=b'z' => ch - b'a' + 26,
            b'0'..=b'9' => ch - b'0' + 52, b'+' => 62, b'/' => 63,
            b'=' => break, _ => continue,
        };
        accumulator = (accumulator << 6) | value as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            bytes.push((accumulator >> bits) as u8);
        }
    }
    bytes
}

fn window_conf() -> Conf {
    Conf { window_title: "RUST // WARS â€” Frontier 09".to_string(), window_width: 1280, window_height: 800, window_resizable: false, ..Default::default() }
}

fn build_frontier_theme() -> Vec<u8> {
    const RATE: u32 = 22_050;
    const BPM: f32 = 108.0;
    const BEAT: f32 = 60.0 / BPM;
    const BEATS: usize = 16;
    const SAMPLES: usize = (RATE as f32 * BEAT * BEATS as f32) as usize;
    const MELODY: [i16; 32] = [
        62, 69, 74, 69, 65, 69, 72, 74, 67, 74, 79, 74, 72, 69, 67, 65,
        62, 69, 74, 77, 76, 74, 72, 69, 65, 69, 72, 76, 74, 72, 69, 65,
    ];
    const BASS: [i16; 4] = [38, 34, 36, 33];
    let mut pcm = Vec::with_capacity(SAMPLES * 2);
    for n in 0..SAMPLES {
        let t = n as f32 / RATE as f32;
        let eighth = BEAT * 0.5;
        let slot = ((t / eighth) as usize) % MELODY.len();
        let slot_time = t % eighth;
        let midi = MELODY[slot];
        let lead = if midi == 0 { 0.0 } else {
            let hz = 440.0 * 2.0_f32.powf((midi as f32 - 69.0) / 12.0);
            let phase = (t * hz).fract();
            let gate = if slot_time > eighth * 0.82 { 0.0 } else { 1.0 };
            let attack = (slot_time / 0.012).min(1.0);
            let release = ((eighth - slot_time) / 0.025).min(1.0);
            (if phase < 0.46 { 1.0 } else { -1.0 }) * gate * attack * release * 0.16
        };

        let beat_index = (t / BEAT) as usize;
        let bar = (beat_index / 4) % BASS.len();
        let bass_note = BASS[bar] + if beat_index % 4 == 2 { 7 } else { 0 };
        let bass_hz = 440.0 * 2.0_f32.powf((bass_note as f32 - 69.0) / 12.0);
        let bass_phase = (t * bass_hz).fract();
        let bass_wave = 1.0 - 4.0 * (bass_phase - 0.5).abs();
        let bass = bass_wave * 0.20;

        let beat_time = t % BEAT;
        let kick = if beat_index % 4 == 0 && beat_time < 0.16 {
            (2.0 * PI * (58.0 + 95.0 * (1.0 - beat_time / 0.16)) * beat_time).sin()
                * (1.0 - beat_time / 0.16) * 0.22
        } else { 0.0 };
        let snare = if beat_index % 4 == 2 && beat_time < 0.08 {
            let noise = (((n as u32).wrapping_mul(1_664_525).wrapping_add(1_013_904_223) >> 16) as i16) as f32 / 32_768.0;
            noise * (1.0 - beat_time / 0.08) * 0.08
        } else { 0.0 };
        let sample = (lead + bass + kick + snare).clamp(-0.92, 0.92);
        pcm.extend_from_slice(&((sample * i16::MAX as f32) as i16).to_le_bytes());
    }

    wav_from_pcm(&pcm, RATE)
}

fn build_sfx(cue: Sfx) -> Vec<u8> {
    const RATE: u32 = 22_050;
    let (duration, start_hz, end_hz, volume) = match cue {
        Sfx::Select => (0.075_f32, 690.0, 940.0, 0.14),
        Sfx::Move => (0.12, 290.0, 180.0, 0.16),
        Sfx::Fire => (0.14, 180.0, 48.0, 0.20),
        Sfx::Impact => (0.19, 92.0, 32.0, 0.20),
        Sfx::Capture => (0.32, 520.0, 820.0, 0.16),
    };
    let samples = (duration * RATE as f32) as usize;
    let mut pcm = Vec::with_capacity(samples * 2);
    for n in 0..samples {
        let t = n as f32 / RATE as f32;
        let progress = n as f32 / samples as f32;
        let hz = start_hz + (end_hz - start_hz) * progress;
        let phase = (t * hz).fract();
        let pulse = if phase < 0.5 { 1.0 } else { -1.0 };
        let triangle = 1.0 - 4.0 * (phase - 0.5).abs();
        let noise = (((n as u32).wrapping_mul(1_103_515_245).wrapping_add(12_345) >> 16) as i16) as f32 / 32_768.0;
        let wave = match cue {
            Sfx::Select | Sfx::Capture => pulse,
            Sfx::Move => triangle,
            Sfx::Fire => pulse * 0.35 + noise * 0.65,
            Sfx::Impact => noise * 0.78 + (2.0 * PI * hz * t).sin() * 0.22,
        };
        let envelope = (1.0 - progress).powf(if cue == Sfx::Capture { 0.9 } else { 1.7 });
        let sample = (wave * envelope * volume).clamp(-0.85, 0.85);
        pcm.extend_from_slice(&((sample * i16::MAX as f32) as i16).to_le_bytes());
    }
    wav_from_pcm(&pcm, RATE)
}

fn wav_from_pcm(pcm: &[u8], rate: u32) -> Vec<u8> {
    let data_len = pcm.len() as u32;
    let mut wav = Vec::with_capacity(44 + pcm.len());
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + data_len).to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt ");
    wav.extend_from_slice(&16_u32.to_le_bytes());
    wav.extend_from_slice(&1_u16.to_le_bytes());
    wav.extend_from_slice(&1_u16.to_le_bytes());
    wav.extend_from_slice(&rate.to_le_bytes());
    wav.extend_from_slice(&(rate * 2).to_le_bytes());
    wav.extend_from_slice(&2_u16.to_le_bytes());
    wav.extend_from_slice(&16_u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_len.to_le_bytes());
    wav.extend_from_slice(pcm);
    wav
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = Game::new();
    let unit_atlas_bytes = decode_base64(include_str!("../assets/unit_atlas.png.b64"));
    let unit_atlas = Texture2D::from_file_with_format(&unit_atlas_bytes, Some(ImageFormat::Png));
    unit_atlas.set_filter(FilterMode::Nearest);
    let music_data = build_frontier_theme();
    let music: Sound = audio::load_sound_from_bytes(&music_data).await.expect("built-in theme is a valid WAV");
    audio::play_sound(&music, PlaySoundParams { looped: true, volume: 0.24 });
    let mut sound_effects = Vec::with_capacity(Sfx::ALL.len());
    for cue in Sfx::ALL {
        let data = build_sfx(cue);
        sound_effects.push(audio::load_sound_from_bytes(&data).await.expect("built-in sound effect is a valid WAV"));
    }
    let mut last_music_on = game.music_on;
    loop {
        game.update();
        for cue in game.sfx_queue.drain(..) {
            audio::play_sound(&sound_effects[cue.index()], PlaySoundParams { looped: false, volume: 0.48 });
        }
        if game.music_on != last_music_on {
            audio::set_sound_volume(&music, if game.music_on { 0.24 } else { 0.0 });
            last_music_on = game.music_on;
        }
        game.draw(&unit_atlas);
        next_frame().await;
    }
}

