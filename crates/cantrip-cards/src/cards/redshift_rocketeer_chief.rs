use cantrip_api::*;
use cantrip_mechanics::{choose, targets};
use cantrip_oracle as oracle;

/// "Add X mana of any one color, where X is Redshift's power. Spend this mana only to activate abilities." (CR 106.5,
/// 106.6): the power as the ability resolves, at once (CR 605.3b); none for a power of 0 or less.
fn power_in_mana(g: &mut dyn Game, asker: &Asker, color: Option<Color>) -> Result<(), Suspend> {
    let power = g.chars(asker.source).power.unwrap_or(0).clamp(0, 255) as u8;
    if let Some(color) = color
        && power > 0
    {
        g.add_restricted_mana(asker.controller, color.mana(), power, ManaRestriction::ABILITIES);
    }
    Ok(())
}

/// "Put any number of permanent cards from your hand onto the battlefield." Chosen as it resolves, and put there together
/// (CR 101.4); an Aura chooses what it enchants as it enters (CR 303.4f).
fn put_permanents(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let eligible: Vec<ObjectId> =
        g.state().zone(Zone::hand(item.controller)).iter().copied().filter(|&c| targets::is_permanent_card(&g.chars(c))).collect();
    // "Any number of" (`u8::MAX`, up to); nothing is asked with no permanent card in the hand.
    let chosen = choose::cards(g, item.controller, item.object, Purpose::PutOntoBattlefield, eligible, u8::MAX, true)?;
    let moves: Vec<(ObjectId, PlayerId)> = chosen.into_iter().map(|c| (c, item.controller)).collect();
    g.put_all_onto_battlefield(&moves)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add_by(power_in_mana).choose_color().only_for(ManaRestriction::ABILITIES), "{T}: Add X mana of any one color, where X is Redshift's power. Spend this mana only to activate abilities."),
        ability(cost("{10}{R}{G}"), put_permanents, "Exhaust \u{2014} {10}{R}{G}: Put any number of permanent cards from your hand onto the battlefield. (Activate each exhaust ability only once.)")
            .exhaust(),
    ],
    ..oracle::REDSHIFT_ROCKETEER_CHIEF
};
