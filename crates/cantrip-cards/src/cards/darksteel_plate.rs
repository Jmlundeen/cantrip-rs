use cantrip_api::*;
use cantrip_mechanics::attach;
use cantrip_oracle as oracle;

/// "Equipped creature has indestructible." (CR 604.1).
fn equipped_creature_has_indestructible_apply_0(
    _: &dyn LayerQuery,
    _: ObjectId,
    chars: &mut Chars,
) {
    chars.keywords = chars.keywords.with(Keyword::Indestructible);
}

/// "Equip {2}" (CR 115.1).
fn equip_2_target_0(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(
        t, Target::Object(id) if cantrip_mechanics::targets::permanent(q, id) && ((q
        .chars(id).is(CardType::Creature) && asker.controller == q.controller(id)))
    )
}

/// "Equip {2}" (CR 608.2, 701.3a).
fn equip_2(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let t_0 = vec![g.state().target(item, 0)];
    let attachments_0 = {
        let q: &dyn Query = g;
        Some(item.object)
            .into_iter()
            .filter(|&o| q.state().obj(o).became.is_none())
            .collect::<Vec<ObjectId>>()
    };
    let host_0 = t_0.get(0).copied().and_then(|t: Target| t.object());
    let Some(host) = host_0 else { return Ok(()) };
    instruction(
        g,
        |g| {
            for attachment in attachments_0 {
                g.attach(attachment, host);
            }
        },
    );
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(cost("{2}"), equip_2, "Equip {2}")
            .targets(&[target(equip_2_target_0)])
            .equip()
            .sorcery_speed(),
    ],
    statics: &[
        static_ability(
            Layer::Ability,
            attach::attached_object,
            equipped_creature_has_indestructible_apply_0,
            "Equipped creature has indestructible.",
        ),
    ],
    ..oracle::DARKSTEEL_PLATE
};
