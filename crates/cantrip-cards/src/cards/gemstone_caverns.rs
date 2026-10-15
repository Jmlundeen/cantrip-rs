use cantrip_api::*;
use cantrip_oracle as oracle;

/// "If this card is in your opening hand and you're not the starting player" (CR 103.6a).
fn not_the_starting_player(q: &dyn Query, asker: &Asker) -> bool {
    q.state().turn.starting != asker.controller
}

fn has_luck(q: &dyn Query, source: ObjectId) -> bool {
    q.obj(source).counters(CounterKind::Luck) > 0
}

/// The colors the printed ability may name: all five while there is a luck counter on the land, none (so it adds {C}) while
/// there is not. The "instead" is part of this one ability (CR 605.1a), so a mana ability the land is granted is its own.
fn lucky_colors(q: &dyn Query, asker: &Asker) -> ColorSet {
    if has_luck(q, asker.source) { ColorSet(0b11111) } else { ColorSet::NONE }
}

/// {T}: Add {C}. If this land has a luck counter on it, instead add one mana of any color.
fn add_mana(g: &mut dyn Game, asker: &Asker, color: Option<Color>) -> Result<(), Suspend> {
    let mana = match color {
        Some(c) if has_luck(g, asker.source) => c.mana(),
        _ => Mana::C,
    };
    g.add_mana(asker.controller, mana, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Activated(ActivatedDef {
        text: "{T}: Add {C}. If Gemstone Caverns has a luck counter on it, instead add one mana of any color.",
        cost: Cost { tap: true, ..Cost::NONE },
        mana: Some(add_mana),
        any_color: true,
        colors_from: Some(lucky_colors),
        // What it adds depends on the counter, so no yield is stated: anything may be payable.
        ..ActivatedDef::BLANK
    })],
    opening_hand: Some(OpeningHandAction {
        condition: not_the_starting_player,
        counter: Some((CounterKind::Luck, 1)),
        exile_from_hand: 1,
    }),
    ..oracle::GEMSTONE_CAVERNS
};
