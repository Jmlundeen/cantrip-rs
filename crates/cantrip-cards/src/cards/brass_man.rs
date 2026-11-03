use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

/// You may pay {1}. If you do, untap this creature (CR 608.2c, 701.26).
fn pay_to_untap(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::may_pay_to_untap_self(g, item, cost("{1}"))
}

pub static CARD: CardDef = CardDef {
    keywords: Keywords::of(&[Keyword::DoesNotUntap]),
    abilities: &[
        triggered(triggers::YOUR_UPKEEP, pay_to_untap, "At the beginning of your upkeep, you may pay {1}. If you do, untap this creature."),
    ],
    ..oracle::BRASS_MAN
};
