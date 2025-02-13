use crate::character::Character;
use crate::game::battle::Attack;
use crate::game::Game;
use crate::item::shop;
use crate::location::Location;
use colored::*;
use once_cell::sync::OnceCell;

// This are initialized based on input args and then act as constants
// this prevents having to pass around the flags or lazily parsing the opts
static QUIET: OnceCell<bool> = OnceCell::new();
static PLAIN: OnceCell<bool> = OnceCell::new();

/// Set the global output preferences
pub fn init(quiet: bool, plain: bool) {
    QUIET.set(quiet).unwrap();
    PLAIN.set(plain).unwrap();
}

fn quiet() -> bool {
    *QUIET.get().unwrap_or(&false)
}

fn plain() -> bool {
    *PLAIN.get().unwrap_or(&false)
}

/// Print the hero status according to options
pub fn status(game: &Game) {
    if plain() {
        plain_status(game);
    } else if quiet() {
        short_status(game);
    } else {
        long_status(game)
    }
}

pub fn enemy_appears(enemy: &Character, location: &Location) {
    log(enemy, location, "");
}

pub fn bribe_success(player: &Character, amount: i32) {
    let suffix = format!("bribed {}", format!("-{}g", amount).yellow());
    battle_log(player, &suffix);
    println!();
}

pub fn bribe_failure(player: &Character) {
    battle_log(player, "can't bribe!");
}

pub fn run_away_success(player: &Character) {
    battle_log(player, "fled!");
}

pub fn run_away_failure(player: &Character) {
    battle_log(player, "can't run!");
}

pub fn tombstone(location: &Location, items: &[String], gold: i32) {
    let name = format!("{:>8}", "hero");
    print!("{}[\u{1FAA6} ]@{}", name, location);
    if gold > 0 {
        print!(" {}", format_gold_plus(gold));
    }
    for item in items {
        print!(" +{}", item);
    }
    println!();
}

pub fn heal(player: &Character, location: &Location, recovered: i32) {
    if recovered > 0 {
        log(
            player,
            location,
            &format!("+{}hp", recovered).green().to_string(),
        );
    }
}

pub fn potion(player: &Character, recovered: i32) {
    if recovered > 0 {
        battle_log(
            player,
            &format!("+{}hp potion", recovered).green().to_string(),
        );
    }
}

pub fn player_attack(enemy: &Character, attack: Attack) {
    if !quiet() {
        battle_log(enemy, &format_attack(attack, "white"));
    }
}

pub fn enemy_attack(player: &Character, attack: Attack) {
    if !quiet() {
        battle_log(player, &format_attack(attack, "bright red"));
    }
}

pub fn battle_lost(player: &Character) {
    battle_log(player, "\u{1F480}");
}

pub fn battle_won(game: &Game, xp: i32, levels_up: i32, gold: i32) {
    let level_str = if levels_up > 0 {
        let plus = (0..levels_up).map(|_| "+").collect::<String>();
        format!(" {}level", plus).cyan().to_string()
    } else {
        "".to_string()
    };

    battle_log(
        &game.player,
        &format!(
            "{}{} {}",
            format!("+{}xp", xp).bold(),
            level_str,
            format_gold_plus(gold)
        ),
    );
    short_status(game);
}

fn long_status(game: &Game) {
    let player = &game.player;
    let location = &game.location;

    println!("{}@{}", format_character(player), location);
    println!(
        "    hp:{} {}/{}",
        hp_display(player, 10),
        player.current_hp,
        player.max_hp
    );
    println!(
        "    xp:{} {}/{}",
        xp_display(player, 10),
        player.xp,
        player.xp_for_next()
    );
    println!(
        "    att:{}   def:{}   spd:{}",
        player.attack(),
        player.deffense(),
        player.speed
    );
    println!("    {}", format_equipment(player));
    println!("    {}", format_inventory(game));
    println!("    {}", format_gold(game.gold));
}

fn short_status(game: &Game) {
    log(&game.player, &game.location, "");
}

fn plain_status(game: &Game) {
    let player = &game.player;
    println!(
        "{}[{}]\t@{}\thp:{}/{}\txp:{}/{}\tatt:{}\tdef:{}\tspd:{}\t{}\t{}\tg:{}",
        player.name(),
        player.level,
        game.location,
        player.current_hp,
        player.max_hp,
        player.xp,
        player.xp_for_next(),
        player.attack(),
        player.deffense(),
        player.speed,
        format_equipment(player),
        format_inventory(game),
        game.gold
    );
}

pub fn shop_list(game: &Game, items: Vec<Box<dyn shop::Shoppable>>) {
    for item in items {
        let display = format!("{}", item);
        println!("    {:<10}  {}", display, format_gold(item.cost()));
    }

    println!("\n    funds: {}", format_gold(game.gold));
}

// HELPERS

/// Generic log function. At the moment all output of the game is structured as
/// of a player status at some location, with an optional event suffix.
fn log(character: &Character, location: &Location, suffix: &str) {
    println!(
        "{}{}{}@{} {}",
        format_character(character),
        hp_display(character, 4),
        xp_display(character, 4),
        location,
        suffix
    );
}

fn battle_log(character: &Character, suffix: &str) {
    println!(
        "{}{} {}",
        format_character(character),
        hp_display(character, 4),
        suffix
    );
}

fn format_character(character: &Character) -> String {
    let name = format!("{:>8}", character.name());
    let name = if character.is_player() {
        name.bold()
    } else {
        name.yellow().bold()
    };
    format!("{}[{}]", name, character.level)
}

fn format_equipment(character: &Character) -> String {
    let mut fragments = Vec::new();

    if let Some(sword) = &character.sword {
        fragments.push(sword.to_string());
    }

    if let Some(shield) = &character.shield {
        fragments.push(shield.to_string());
    }
    format!("equip:{{{}}}", fragments.join(","))
}

pub fn format_inventory(game: &Game) -> String {
    let mut items = game
        .inventory()
        .iter()
        .map(|(k, v)| format!("{}x{}", k, v))
        .collect::<Vec<String>>();

    items.sort();
    format!("item:{{{}}}", items.join(","))
}

fn format_attack(attack: Attack, color: &str) -> String {
    match attack {
        Attack::Regular(damage) => format!("-{}hp", damage).color(color).to_string(),
        Attack::Critical(damage) => format!("-{}hp critical!", damage).color(color).to_string(),
        Attack::Miss => " dodged!".to_string(),
    }
}

fn hp_display(character: &Character, slots: i32) -> String {
    bar_display(
        slots,
        character.current_hp,
        character.max_hp,
        "green",
        "red",
    )
}

fn xp_display(character: &Character, slots: i32) -> String {
    if character.is_player() {
        bar_display(
            slots,
            character.xp,
            character.xp_for_next(),
            "cyan",
            "bright black",
        )
    } else {
        // enemies don't have experience
        String::new()
    }
}

fn bar_display(
    slots: i32,
    current: i32,
    total: i32,
    current_color: &str,
    missing_color: &str,
) -> String {
    let (filled, rest) = bar_slots(slots, total, current);
    let current = (0..filled)
        .map(|_| "x")
        .collect::<String>()
        .color(current_color);
    let missing = (0..rest)
        .map(|_| "-")
        .collect::<String>()
        .color(missing_color);
    format!("[{}{}]", current, missing)
}

fn bar_slots(slots: i32, total: i32, current: i32) -> (i32, i32) {
    let units = (current as f64 * slots as f64 / total as f64).ceil() as i32;
    (units, slots - units)
}

fn format_gold(gold: i32) -> ColoredString {
    format!("{}g", gold).yellow()
}

fn format_gold_plus(gold: i32) -> ColoredString {
    format!("+{}g", gold).yellow()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bar_slots() {
        // simple case 1:1 between points and slots
        let slots = 4;
        let total = 4;
        assert_eq!((0, 4), bar_slots(slots, total, 0));
        assert_eq!((1, 3), bar_slots(slots, total, 1));
        assert_eq!((2, 2), bar_slots(slots, total, 2));
        assert_eq!((3, 1), bar_slots(slots, total, 3));
        assert_eq!((4, 0), bar_slots(slots, total, 4));

        let total = 10;
        assert_eq!((0, 4), bar_slots(slots, total, 0));
        assert_eq!((1, 3), bar_slots(slots, total, 1));
        assert_eq!((1, 3), bar_slots(slots, total, 2));
        assert_eq!((2, 2), bar_slots(slots, total, 3));
        assert_eq!((2, 2), bar_slots(slots, total, 4));
        assert_eq!((2, 2), bar_slots(slots, total, 5));
        assert_eq!((3, 1), bar_slots(slots, total, 6));
        assert_eq!((3, 1), bar_slots(slots, total, 7));
        // this one I would maybe like to show as 3, 1
        assert_eq!((4, 0), bar_slots(slots, total, 8));
        assert_eq!((4, 0), bar_slots(slots, total, 9));
        assert_eq!((4, 0), bar_slots(slots, total, 10));
    }
}
