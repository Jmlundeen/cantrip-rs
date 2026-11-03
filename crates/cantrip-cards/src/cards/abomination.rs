use cantrip_api::*;
use cantrip_mechanics::delayed;
use cantrip_oracle as oracle;

/// Each creature this blocks or that blocks it is checked for green or white as the block happens (CR 509.3b, 509.3d).
fn blocks_or_becomes_blocked(q: &dyn Query, asker: &Asker, event: &Event) -> Option<Fired> {
    let Event::Blocks { blocker, attacker, .. } = event else { return None };
    let other = if *blocker == asker.source {
        *attacker
    } else if *attacker == asker.source {
        *blocker
    } else {
        return None;
    };
    let colors = q.chars(other).colors;
    let words = q.words(asker.source);
    if colors.has(words.color(Color::Green)) || colors.has(words.color(Color::White)) {
        Fired::about(Target::Object(other))
    } else {
        None
    }
}

const TEXT: &str = "Whenever this creature blocks or becomes blocked by a green or white creature, destroy that creature at end of combat.";

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(when_asked(EventMask::BLOCKS, blocks_or_becomes_blocked), delayed::destroy_that_creature_at_end_of_combat, TEXT)],
    delayed: &[delayed::DESTROY_THAT_CREATURE_AT_END_OF_COMBAT],
    ..oracle::ABOMINATION
};
