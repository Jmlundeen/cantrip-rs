use cantrip_api::*;
use cantrip_oracle as oracle;

/// "You may have this enchantment enter as a copy of any artifact on the battlefield, except it's an enchantment in
/// addition to its other types." A choice, not a target (CR 608.2d). The exception is part of the copy's own copiable
/// values (CR 707.9b), so a later copy of it is an enchantment too. Declined, or with no artifact to copy, it enters as
/// itself, an enchantment with no abilities (the card's ruling).
fn enter_as_copy(g: &mut dyn Game, entry: &mut Entering) -> Result<(), Suspend> {
    cantrip_mechanics::copy::enter_as_copy(
        g,
        entry,
        |c| c.is(CardType::Artifact),
        |_, copy| copy.with_type(CardType::Enchantment),
    )
}

pub static CARD: CardDef = CardDef { enters_as_copy: Some(enter_as_copy), ..oracle::COPY_ARTIFACT };
