use cantrip_api::*;
use cantrip_mechanics::library::search_aura_or_equipment_to_hand;
use cantrip_mechanics::statics;
use cantrip_oracle as oracle;

/// The Adventure (CR 715): Search your library for an Aura or Equipment card, reveal it, put it into your hand, then shuffle.
fn birthright_boon(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    search_aura_or_equipment_to_hand(g, item.controller, item.object)?;
    Ok(())
}

pub static BIRTHRIGHT_BOON: CardDef = CardDef { spell: spell(&[], birthright_boon), ..oracle::BIRTHRIGHT_BOON };

/// "+1/+0 for each Aura and Equipment attached to Kellan" (CR 613.4c, layer 7c): counted as the layers are worked out, so it
/// follows each attach and each Aura or Equipment that leaves. Each attached permanent counts once, judged on what the
/// earlier layers made it.
fn plus_one_per_attachment(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let n = view
        .state()
        .battlefield()
        .iter()
        .filter(|&&id| {
            view.state().obj(id).attached_to == Some(source) && {
                let c = view.chars(id);
                c.has_subtype(Subtype::Aura) || c.has_subtype(Subtype::Equipment)
            }
        })
        .count() as i16;
    chars.power = chars.power.map(|v| v + n);
}

pub static CARD: CardDef = CardDef {
    statics: &[static_ability(
        Layer::PtModify,
        statics::other_creatures_you_control,
        plus_one_per_attachment,
        "Other creatures you control get +1/+0 for each Aura and Equipment attached to Kellan.",
    )],
    adventure: Some(&BIRTHRIGHT_BOON),
    ..oracle::KELLAN_THE_FAE_BLOODED
};

pub static EXTRA: &[&CardDef] = &[&BIRTHRIGHT_BOON];
