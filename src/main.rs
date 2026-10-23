use std::io::{self, Write};

const WIDTH: usize = 10;
const HEIGHT: usize = 8;
const HUMAN: u8 = 0;
const BOTS: u8 = 1;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Terrain { Plain, Forest, Mountain }

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind { Infantry, Armor, Artillery }

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
    terrain: [[Terrain; WIDTH]; HEIGHT],
    units: Vec<Unit>,
    sites: Vec<Site>,
    selected: Option<usize>,
    round: usize,
    over: bool,
}

impl Game {
    fn new() -> Self {
        let mut terrain = [[Terrain::Plain; WIDTH]; HEIGHT];
        for (x, y) in [(2,1),(3,1),(7,1),(1,3),(5,3),(8,3),(3,5),(6,6),(8,6)] {
            terrain[y][x] = Terrain::Forest;
        }
        for (x, y) in [(4,2),(5,2),(4,5),(5,5),(6,5)] { terrain[y][x] = Terrain::Mountain; }
        let units = vec![
            Unit { id: 1, team: HUMAN, kind: Kind::Infantry, x: 1, y: 6, hp: 10, acted: false },
            Unit { id: 2, team: HUMAN, kind: Kind::Armor, x: 2, y: 7, hp: 10, acted: false },
            Unit { id: 3, team: HUMAN, kind: Kind::Artillery, x: 0, y: 7, hp: 10, acted: false },
            Unit { id: 4, team: BOTS, kind: Kind::Infantry, x: 8, y: 1, hp: 10, acted: false },
            Unit { id: 5, team: BOTS, kind: Kind::Armor, x: 8, y: 0, hp: 10, acted: false },
            Unit { id: 6, team: BOTS, kind: Kind::Artillery, x: 9, y: 0, hp: 10, acted: false },
        ];
        let sites = vec![
            Site { x: 1, y: 2, owner: None }, Site { x: 4, y: 3, owner: None },
            Site { x: 7, y: 4, owner: None }, Site { x: 8, y: 7, owner: None },
        ];
        Self { terrain, units, sites, selected: None, round: 1, over: false }
    }

    fn render(&self) {
        print!("\x1b[2J\x1b[H");
        println!("╔══════════════════════════════════════════════════════╗");
        println!("║  R U S T   W A R S       FRONTIER 09        DEMO    ║");
        println!("╚══════════════════════════════════════════════════════╝");
        println!("  ROUND {:02}   HUMAN SITES {}/3   BOT SITES {}/3", self.round,
            self.sites.iter().filter(|s| s.owner == Some(HUMAN)).count(),
            self.sites.iter().filter(|s| s.owner == Some(BOTS)).count());
        print!("     "); for x in 0..WIDTH { print!(" {} ", x); } println!();
        print!("    ┌"); for _ in 0..WIDTH { print!("───"); } println!("┐");
        for y in 0..HEIGHT {
            print!(" {:2} │", y);
            for x in 0..WIDTH {
                let unit = self.units.iter().find(|u| u.x == x && u.y == y);
                if let Some(u) = unit {
                    let glyph = match (u.team, u.kind) {
                        (HUMAN, Kind::Infantry) => "H", (HUMAN, Kind::Armor) => "T", (HUMAN, Kind::Artillery) => "A",
                        (_, Kind::Infantry) => "b", (_, Kind::Armor) => "t", (_, Kind::Artillery) => "a",
                    };
                    let color = if u.team == HUMAN { "\x1b[96m" } else { "\x1b[91m" };
                    let marker = if self.selected == Some(u.id) { "*" } else { " " };
                    print!("{}{}{}\x1b[0m", color, glyph, marker);
                } else if let Some(site) = self.sites.iter().find(|s| s.x == x && s.y == y) {
                    let (icon, color) = match site.owner { Some(HUMAN) => ("⌂", "\x1b[96m"), Some(BOTS) => ("⌂", "\x1b[91m"), _ => ("⌂", "\x1b[93m") };
                    print!("{} {}\x1b[0m", color, icon);
                } else {
                    let (icon, color) = match self.terrain[y][x] {
                        Terrain::Plain => ("·", "\x1b[90m"), Terrain::Forest => ("♣", "\x1b[92m"), Terrain::Mountain => ("▲", "\x1b[37m"),
                    };
                    print!("{} {}\x1b[0m", color, icon);
                }
            }
            println!("│");
        }
        print!("    └"); for _ in 0..WIDTH { print!("───"); } println!("┘");
        println!("  HUMAN  H infantry  T armor  A artillery      BOT  b / t / a");
        println!("  SITE   ⌂ command post       FOREST ♣ cover      MOUNTAIN ▲ blocked");
        if let Some(id) = self.selected {
            if let Some(u) = self.units.iter().find(|u| u.id == id) {
                println!("\n  Selected: #{} {}  HP {}  at ({}, {})  {}", u.id, kind_name(u.kind), u.hp, u.x, u.y,
                    if u.acted { "ACTION SPENT" } else { "READY" });
            }
        }
    }

    fn command(&mut self, line: &str) -> bool {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() { return true; }
        match parts[0].to_lowercase().as_str() {
            "help" => help(),
            "quit" | "q" => return false,
            "units" => self.list_units(),
            "select" if parts.len() == 2 => match parts[1].parse::<usize>() {
                Ok(id) if self.units.iter().any(|u| u.id == id && u.team == HUMAN) => { self.selected = Some(id); }
                _ => println!("No human unit has that ID."),
            },
            "move" if parts.len() == 3 => if let (Some(id), Ok(x), Ok(y)) = (self.selected, parts[1].parse::<usize>(), parts[2].parse::<usize>()) { self.move_unit(id, x, y); } else { println!("Select a unit and enter valid coordinates."); },
            "attack" if parts.len() == 3 => if let (Some(id), Ok(x), Ok(y)) = (self.selected, parts[1].parse::<usize>(), parts[2].parse::<usize>()) { self.attack(id, x, y); } else { println!("Select a unit and enter valid coordinates."); },
            "end" | "endturn" => self.end_turn(),
            _ => println!("Unknown command. Type help for orders."),
        }
        self.check_victory();
        true
    }

    fn list_units(&self) {
        println!("\n  ID  SIDE   UNIT       HP  POSITION  STATE");
        for u in &self.units { println!("  {:<3} {:<6} {:<10} {:<3} ({}, {})   {}", u.id,
            if u.team == HUMAN { "HUMAN" } else { "BOT" }, kind_name(u.kind), u.hp, u.x, u.y,
            if u.acted { "SPENT" } else { "READY" }); }
    }

    fn move_unit(&mut self, id: usize, x: usize, y: usize) {
        if x >= WIDTH || y >= HEIGHT { println!("That coordinate is outside the map."); return; }
        let idx = match self.units.iter().position(|u| u.id == id && u.team == HUMAN) { Some(i) => i, None => { println!("Unit unavailable."); return; } };
        let u = &self.units[idx];
        if u.acted { println!("That unit has already acted this round."); return; }
        let distance = u.x.abs_diff(x) + u.y.abs_diff(y);
        let allowance = if u.kind == Kind::Armor { 4 } else { 3 };
        if distance == 0 || distance > allowance { println!("Destination is outside this unit's movement range."); return; }
        if self.terrain[y][x] == Terrain::Mountain || self.units.iter().any(|other| other.x == x && other.y == y) { println!("Destination is blocked."); return; }
        if self.terrain[y][x] == Terrain::Forest && distance > allowance - 1 { println!("Forest slows movement; choose a closer tile."); return; }
        self.units[idx].x = x; self.units[idx].y = y; self.units[idx].acted = true;
        self.capture(x, y, HUMAN, self.units[idx].kind);
    }

    fn attack(&mut self, id: usize, x: usize, y: usize) {
        let idx = match self.units.iter().position(|u| u.id == id && u.team == HUMAN) { Some(i) => i, None => { println!("Unit unavailable."); return; } };
        let (ux, uy, kind, acted) = { let u = &self.units[idx]; (u.x, u.y, u.kind, u.acted) };
        if acted { println!("That unit has already acted this round."); return; }
        let target_idx = match self.units.iter().position(|u| u.team == BOTS && u.x == x && u.y == y) { Some(i) => i, None => { println!("No bot unit at that coordinate."); return; } };
        let distance = ux.abs_diff(x) + uy.abs_diff(y);
        let in_range = match kind { Kind::Artillery => (2..=3).contains(&distance), _ => distance == 1 };
        if !in_range { println!("Target is outside weapon range."); return; }
        let base = damage(kind, self.units[target_idx].kind);
        let cover = if self.terrain[y][x] == Terrain::Forest { 1 } else { 0 };
        let dealt = (base - cover).max(1);
        self.units[target_idx].hp -= dealt;
        println!("Unit #{} hits bot #{} for {} damage!", id, self.units[target_idx].id, dealt);
        self.units[idx].acted = true;
        if self.units[target_idx].hp <= 0 { println!("Bot unit #{} destroyed.", self.units[target_idx].id); self.units.remove(target_idx); }
    }

    fn capture(&mut self, x: usize, y: usize, team: u8, kind: Kind) {
        if kind != Kind::Infantry { return; }
        if let Some(site) = self.sites.iter_mut().find(|s| s.x == x && s.y == y) {
            if site.owner != Some(team) { site.owner = Some(team); println!("Command site captured by {}!", if team == HUMAN { "HUMAN FORCES" } else { "THE WAR BOTS" }); }
        }
    }

    fn end_turn(&mut self) {
        for u in &mut self.units { if u.team == HUMAN { u.acted = true; } }
        let bots: Vec<usize> = self.units.iter().filter(|u| u.team == BOTS).map(|u| u.id).collect();
        for id in bots {
            if let Some(idx) = self.units.iter().position(|u| u.id == id) { self.bot_action(idx); }
            self.check_victory(); if self.over { return; }
        }
        self.round += 1; self.selected = None;
        for u in &mut self.units { u.acted = false; }
    }

    fn bot_action(&mut self, idx: usize) {
        let (id, kind, x, y) = { let u = &self.units[idx]; (u.id, u.kind, u.x, u.y) };
        let targets: Vec<(usize, usize)> = self.units.iter().filter(|u| u.team == HUMAN).map(|u| (u.x,u.y)).collect();
        if let Some((tx,ty)) = targets.into_iter().filter(|(tx,ty)| {
            let d = x.abs_diff(*tx) + y.abs_diff(*ty);
            if kind == Kind::Artillery { (2..=3).contains(&d) } else { d == 1 }
        }).min_by_key(|(tx,ty)| x.abs_diff(*tx)+y.abs_diff(*ty)) {
            if let Some(ti) = self.units.iter().position(|u| u.team == HUMAN && u.x == tx && u.y == ty) {
                let dealt = (damage(kind, self.units[ti].kind) - if self.terrain[ty][tx] == Terrain::Forest { 1 } else { 0 }).max(1);
                self.units[ti].hp -= dealt;
                println!("Bot #{} fires: {} damage to human #{}.", id, dealt, self.units[ti].id);
                if self.units[ti].hp <= 0 { println!("Human unit #{} lost!", self.units[ti].id); self.units.remove(ti); }
                return;
            }
        }
        let goal = self.sites.iter().filter(|s| s.owner != Some(BOTS)).min_by_key(|s| x.abs_diff(s.x)+y.abs_diff(s.y)).map(|s| (s.x,s.y));
        if let Some((gx,gy)) = goal {
            let mut options = Vec::new();
            for (nx,ny) in neighbors(x,y) {
                if nx >= WIDTH || ny >= HEIGHT || self.terrain[ny][nx] == Terrain::Mountain || self.units.iter().any(|u| u.x == nx && u.y == ny) { continue; }
                let cost = x.abs_diff(nx)+y.abs_diff(ny) + if self.terrain[ny][nx] == Terrain::Forest { 1 } else { 0 };
                if cost <= if kind == Kind::Armor { 4 } else { 3 } { options.push((nx.abs_diff(gx)+ny.abs_diff(gy), nx, ny)); }
            }
            if let Some((_, nx, ny)) = options.into_iter().min_by_key(|o| o.0) {
                self.units[idx].x=nx; self.units[idx].y=ny;
                self.capture(nx,ny,BOTS,kind);
            }
        }
    }

    fn check_victory(&mut self) {
        let human_sites = self.sites.iter().filter(|s| s.owner == Some(HUMAN)).count();
        let bot_sites = self.sites.iter().filter(|s| s.owner == Some(BOTS)).count();
        let humans = self.units.iter().any(|u| u.team == HUMAN);
        let bots = self.units.iter().any(|u| u.team == BOTS);
        if human_sites >= 3 || (!bots && humans) { self.over = true; println!("\n  ✦ VICTORY! The frontier belongs to humanity. ✦"); }
        else if bot_sites >= 3 || (!humans && bots) { self.over = true; println!("\n  ☠ DEFEAT. The war bots overran the frontier. ☠"); }
    }
}

fn neighbors(x: usize, y: usize) -> [(usize,usize);4] { [(x.saturating_sub(1),y),(x+1,y),(x,y.saturating_sub(1)),(x,y+1)] }
fn kind_name(k: Kind) -> &'static str { match k { Kind::Infantry => "Infantry", Kind::Armor => "Armor", Kind::Artillery => "Artillery" } }
fn damage(a: Kind, d: Kind) -> i32 { match (a,d) { (Kind::Armor,Kind::Infantry) => 7, (Kind::Armor,_) => 5, (Kind::Infantry,Kind::Armor) => 2, (Kind::Infantry,_) => 4, (Kind::Artillery,Kind::Armor) => 6, (Kind::Artillery,_) => 5 } }
fn help() {
    println!("\n  COMMANDS");
    println!("  select <id>       select one of your units");
    println!("  move <x> <y>      move the selected unit");
    println!("  attack <x> <y>    attack an enemy in range");
    println!("  units             show unit IDs, HP and positions");
    println!("  end               let the war bots take their turn");
    println!("  help / quit       commands / leave the demo");
}

fn main() {
    let mut game = Game::new();
    help();
    while !game.over {
        game.render();
        print!("\n  COMMANDER > "); io::stdout().flush().unwrap();
        let mut line = String::new();
        if io::stdin().read_line(&mut line).is_err() { break; }
        if !game.command(&line) { break; }
        if !game.over { println!("\n  Press Enter to continue..."); let _ = io::stdin().read_line(&mut String::new()); }
    }
    println!("\n  Session ended. Long live the frontier.\n");
}
