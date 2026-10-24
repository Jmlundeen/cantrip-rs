use cantrip_api::*;
use cantrip_mechanics::{history, tokens};
use cantrip_oracle as oracle;

/// "Activate only if this land entered this turn" (CR 602.5b): this object, the one its move onto the battlefield made
/// (CR 400.7), asked of the event history.
fn entered_this_turn(q: &dyn Query, asker: &Asker) -> bool {
    history::entered_this_turn(q.history(), asker.source)
}

/// "Create a 1/1 colorless Phyrexian Mite artifact creature token with toxic 1 and 'This token can't block.'"
fn make_mite(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::PHYREXIAN_MITE, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        mana_ability(TAP, add_any_color(1), "{T}: Add one mana of any color. Activate only if this land entered this turn.")
            .condition(entered_this_turn),
        ability(
            cost("{3}, {T}"),
            make_mite,
            "{3}, {T}: Create a 1/1 colorless Phyrexian Mite artifact creature token with toxic 1 and \"This token can't block.\" (Players dealt combat damage by it also get a poison counter.)",
        ),
    ],
    ..oracle::MIRREX
};

pub static EXTRA: &[&CardDef] = &[&tokens::PHYREXIAN_MITE];
