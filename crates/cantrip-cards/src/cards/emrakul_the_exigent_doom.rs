use cantrip_api::*;
use cantrip_mechanics::land_until_cast;
use cantrip_mechanics::triggers::{CAST_THIS_SPELL, ward_sacrifice};
use cantrip_oracle as oracle;

/// "When you cast this spell, untap all lands you control" (CR 603.2, 701.26b): every land its controller controls as the
/// ability resolves, with no choice. It triggers from the stack, wherever the spell was cast from.
fn untap_lands(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let lands: Vec<ObjectId> = g.permanents_of(item.controller).into_iter().filter(|&id| g.chars(id).is(CardType::Land)).collect();
    for land in lands {
        g.untap(land);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(CAST_THIS_SPELL, untap_lands, "When you cast this spell, untap all lands you control.").anywhere(),
        ward_sacrifice::<3>("Ward\u{2014}Sacrifice three permanents."),
        land_until_cast::ability::<0>(
            cost("{3}"),
            "{3}, Exile this card from your hand: Target land gains \"{T}: Add {C}{C}\" until this card is cast from exile. You may cast this card for as long as it remains exiled.",
        ),
    ],
    delayed: &[mana_ability(TAP, add_n(Mana::C, 2), "{T}: Add {C}{C}.")],
    ..oracle::EMRAKUL_THE_EXIGENT_DOOM
};
