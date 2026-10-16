use cantrip_api::*;
use cantrip_mechanics::tokens::{self, GNOME};
use cantrip_mechanics::triggers::THIS_ENTERS_OR_ATTACKS;
use cantrip_mechanics::{choose, craft};
use cantrip_oracle as oracle;

/// Whether a permanent is a double-faced artifact: an artifact represented by a double-faced card, or a token copy of one
/// (CR 712.1, 111.13), which is the card it copies with a back face of its own. A permanent that is only a copy of a
/// double-faced permanent is not double-faced itself (the Golden-Gear Colossus ruling).
fn double_faced_artifact(q: &dyn Query, id: ObjectId) -> bool {
    q.chars(id).is(CardType::Artifact) && q.card(q.obj(id).card).back.is_some()
}

/// "Whenever Tetzin or another double-faced artifact you control enters" (CR 603.6a): Tetzin itself, or another such artifact
/// entering under its controller's control.
fn tetzin_or_double_faced_artifact_enters(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { new, to, .. }
            if to.is(ZoneKind::Battlefield)
                && (*new == me.source || (q.controller(*new) == me.controller && double_faced_artifact(q, *new))) =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

/// "Mill three cards. You may put an artifact card from among them into your hand." (CR 701.17) The cards milled are the
/// ones it may take from, as they are in the graveyard; choosing none declines.
fn mill_three_take_artifact(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let milled = g.mill(item.controller, 3);
    let artifacts: Vec<ObjectId> = milled.iter().copied().filter(|&c| g.chars(c).is(CardType::Artifact)).collect();
    let chosen = choose::cards_from_kind(g, item.controller, item.object, Purpose::PutIntoHand, "artifact", milled, artifacts, 1, true)?;
    g.return_to_hand(&chosen);
    Ok(())
}

/// "Craft with six artifacts": six from among other permanents you control and/or cards from your graveyard (CR 702.167b).
static SIX_ARTIFACTS: Materials = craft::materials(6, false, craft::artifact, "artifact");

/// "Up to one other target double-faced artifact you control."
fn other_double_faced_artifact_you_control(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id)
        if id != asker.source && cantrip_mechanics::targets::permanent(q, id) && q.controller(id) == asker.controller
            && double_faced_artifact(q, id))
}

/// "Transform up to one other target double-faced artifact you control. Create two 1/1 colorless Gnome artifact creature
/// tokens." Only a transforming double-faced permanent turns over (CR 701.28c, the card's ruling); the Gnomes come either
/// way.
fn transform_and_gnomes(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(artifact) = cantrip_mechanics::targets::object(g, item, 0) {
        g.transform(artifact);
    }
    tokens::create(g, item.controller, &GNOME, 2)?;
    Ok(())
}

pub static BACK: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS_OR_ATTACKS, transform_and_gnomes, "Whenever The Golden-Gear Colossus enters or attacks, transform up to one other target double-faced artifact you control. Create two 1/1 colorless Gnome artifact creature tokens.")
        .targets(&[up_to_one(other_double_faced_artifact_you_control)])],
    ..oracle::THE_GOLDEN_GEAR_COLOSSUS
};

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(when_asked(EventMask::ZONE_CHANGE, tetzin_or_double_faced_artifact_enters), mill_three_take_artifact, "Whenever Tetzin or another double-faced artifact you control enters, mill three cards. You may put an artifact card from among them into your hand."),
        craft::ability(mana("{4}"), &SIX_ARTIFACTS, "Craft with six artifacts {4}"),
    ],
    back: Some(&BACK),
    ..oracle::TETZIN_GNOME_CHAMPION
};

pub static EXTRA: &[&CardDef] = &[&GNOME];
