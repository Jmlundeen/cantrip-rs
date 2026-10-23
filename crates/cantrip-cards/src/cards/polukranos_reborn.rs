use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// The 3/3 green and white Phyrexian Hydra creature token with reach (CR 111.3, 111.4).
pub static HYDRA_REACH: CardDef = CardDef {
    colors: Some(ColorSet(ColorSet::of(Color::Green).0 | ColorSet::of(Color::White).0)),
    types: TypeSet::of(&[CardType::Creature]),
    power: Some(3),
    toughness: Some(3),
    keywords: Keywords::of(&[Keyword::Reach]),
    ..token!(Phyrexian Hydra)
};

/// The 3/3 green and white Phyrexian Hydra creature token with lifelink.
pub static HYDRA_LIFELINK: CardDef = CardDef {
    keywords: Keywords::of(&[Keyword::Lifelink]),
    ..HYDRA_REACH
};

/// "Whenever Polukranos or another nontoken Hydra you control dies" (CR 700.4, 603.10a), looked back on: each Hydra as it
/// last was on the battlefield, under the controller it last had, so Hydras that die together trigger it once each.
/// "Nontoken" qualifies only "another Hydra": Polukranos itself triggers it whatever it is, a token copy included.
fn hydra_dies(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { old, from, to, .. }
            if from.is(ZoneKind::Battlefield)
                && to.is(ZoneKind::Graveyard)
                && q.obj(*old).controller == me.controller
                && (*old == me.source
                    || (!q.obj(*old).token && q.chars(*old).has_subtype(q.words(me.source).subtype(Subtype::Hydra)))) =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

/// "Create a 3/3 green and white Phyrexian Hydra creature token with reach and a 3/3 green and white Phyrexian Hydra
/// creature token with lifelink": one instruction, so the two enter together (CR 101.4).
fn create_hydras(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let reach = g.card_id(&HYDRA_REACH);
    let lifelink = g.card_id(&HYDRA_LIFELINK);
    g.create_tokens_together(&[(reach, item.controller), (lifelink, item.controller)])?;
    Ok(())
}

pub static BACK: CardDef = CardDef {
    abilities: &[triggered(when_asked(EventMask::ZONE_CHANGE, hydra_dies), create_hydras, "Whenever Polukranos or another nontoken Hydra you control dies, create a 3/3 green and white Phyrexian Hydra creature token with reach and a 3/3 green and white Phyrexian Hydra creature token with lifelink.")],
    ..oracle::POLUKRANOS_ENGINE_OF_RUIN
};

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{6}{W/P}"), effects::transform_self, "{6}{W/P}: Transform Polukranos Reborn. Activate only as a sorcery.").sorcery_speed()],
    back: Some(&BACK),
    ..oracle::POLUKRANOS_REBORN
};

pub static EXTRA: &[&CardDef] = &[&HYDRA_REACH, &HYDRA_LIFELINK];
