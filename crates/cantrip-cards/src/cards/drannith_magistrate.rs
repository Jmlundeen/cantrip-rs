use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Your opponents can't cast spells from anywhere other than their hands" (CR 101.2, 601.3): a cast rule, true for who may
/// cast from where. A permission to cast from elsewhere (flashback, an airbended card, an Adventure) is overruled; playing a
/// land and activating abilities are not casting, and nor is putting a card onto the battlefield.
fn opponents_cast_only_from_hand(_: &dyn Query, asker: &Asker, caster: PlayerId, from: ZoneKind) -> bool {
    caster == asker.controller || from == ZoneKind::Hand
}

pub static CARD: CardDef = CardDef { cast_rules: &[opponents_cast_only_from_hand], ..oracle::DRANNITH_MAGISTRATE };
