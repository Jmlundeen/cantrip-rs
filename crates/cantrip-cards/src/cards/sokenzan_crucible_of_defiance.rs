use cantrip_api::*;
use cantrip_mechanics::{channel, haste_until_end_of_turn, tokens};
use cantrip_oracle as oracle;

/// "Create two 1/1 colorless Spirit creature tokens. They gain haste until end of turn" (CR 111.1, 611.2): both tokens,
/// created together, gain haste from this ability's source.
fn two_hasty_spirits(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let spirits = tokens::create(g, item.controller, &tokens::SPIRIT_1_1_COLORLESS, 2)?;
    g.end_sentence();
    instruction(g, |g| {
        for spirit in spirits {
            haste_until_end_of_turn(g, item.object, spirit);
        }
    });
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::R), "{T}: Add {R}."),
        ability(cost("{3}{R}").and(DISCARD_THIS), two_hasty_spirits, "Channel \u{2014} {3}{R}, Discard this card: Create two 1/1 colorless Spirit creature tokens. They gain haste until end of turn. This ability costs {1} less to activate for each legendary creature you control.")
            .from(ZoneKind::Hand)
            .cost_change(channel::cheaper_per_legendary_creature),
    ],
    ..oracle::SOKENZAN_CRUCIBLE_OF_DEFIANCE
};

pub static EXTRA: &[&CardDef] = &[&tokens::SPIRIT_1_1_COLORLESS];
