use cantrip_api::*;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// "Artifact creatures you control have "Whenever this creature attacks, each opponent loses 1 life and you gain 1 life."" (CR 604.1).
fn artifact_creatures_control_have_whenever_affects(
    q: &dyn LayerQuery,
    source: ObjectId,
    affected: ObjectId,
    so_far: &Chars,
) -> bool {
    let asker = &Asker::new(source, q.controller(source));
    (*so_far).is(CardType::Artifact) && (*so_far).is(CardType::Creature)
        && asker.controller == q.controller(affected)
}

/// "Whenever this creature attacks, each opponent loses 1 life and you gain 1 life." (CR 608.2, 119.3).
fn granted_0(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.each_opponent_loses_life(item.controller, 1);
    g.end_sentence();
    g.gain_life(item.controller, 1);
    Ok(())
}

/// "Artifact creatures you control have "Whenever this creature attacks, each opponent loses 1 life and you gain 1 life."" (CR 604.1).
fn artifact_creatures_control_have_whenever_apply_0(
    q: &dyn LayerQuery,
    source: ObjectId,
    chars: &mut Chars,
) {
    let asker = &Asker::new(source, q.controller(source));
    let card = {
        let o = q.state().obj(asker.source);
        o.copied_card().unwrap_or(o.card)
    };
    let _ = card;
    let Some(slot) = chars.abilities.iter().position(|a| a.is_none()) else { return };
    chars.abilities[slot] = Some(
        AbilityRef::delayed(
            {
                let o = q.state().obj(asker.source);
                o.copied_card().unwrap_or(o.card)
            },
            0,
        ),
    );
}

pub static CARD: CardDef = CardDef {
    statics: &[
        static_ability(
            Layer::Ability,
            artifact_creatures_control_have_whenever_affects,
            artifact_creatures_control_have_whenever_apply_0,
            "Artifact creatures you control have \"Whenever this creature attacks, each opponent loses 1 life and you gain 1 life.\"",
        ),
    ],
    delayed: &[
        triggered(
            triggers::THIS_ATTACKS,
            granted_0,
            "Whenever this creature attacks, each opponent loses 1 life and you gain 1 life.",
        ),
    ],
    ..oracle::BIO_ASSET_ALLOCATOR
};
