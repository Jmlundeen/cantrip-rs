use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// You may pay {4}. If you do, untap this artifact (CR 608.2c, 701.26).
fn pay_to_untap(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::may_pay_to_untap_self(g, item, cost("{4}"))
}

/// It deals 1 damage to you: dealt by the permanent, and still dealt by one that has left, off the status it last had.
fn burn(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.damage(item.object, Target::Player(item.controller), 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    // "This artifact doesn't untap during your untap step."
    keywords: Keywords::of(&[Keyword::DoesNotUntap]),
    abilities: &[
        mana_ability(TAP, add_n(Mana::C, 3), "{T}: Add {C}{C}{C}."),
        triggered(triggers::YOUR_UPKEEP, pay_to_untap, "At the beginning of your upkeep, you may pay {4}. If you do, untap this artifact."),
        triggered(triggers::YOUR_DRAW_STEP, burn, "At the beginning of your draw step, if this artifact is tapped, it deals 1 damage to you.")
            .condition(triggers::this_is_tapped),
    ],
    ..oracle::MANA_VAULT
};
