use cantrip_api::*;
use cantrip_mechanics::library::exile_top_may_play;
use cantrip_mechanics::mana::add_u;
use cantrip_mechanics::tokens::{self, CONSTRUCT};
use cantrip_mechanics::triggers::this_enters;
use cantrip_oracle as oracle;

/// When Urza enters, create a 0/0 colorless Construct artifact creature token with "This token gets +1/+1 for each artifact
/// you control."
fn make_construct(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &CONSTRUCT, 1)?;
    Ok(())
}

fn untapped_artifact_you_control(q: &dyn Query, asker: &Asker, id: ObjectId) -> bool {
    q.controller(id) == asker.controller && !q.obj(id).tapped && q.chars(id).is(CardType::Artifact)
}

/// Shuffle your library, then exile the top card. Until end of turn, you may play that card without paying its mana cost.
fn play_top(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    g.shuffle(player);
    exile_top_may_play(
        g,
        player,
        1,
        player,
        PlayGrant { instead: Some(ManaCost::ZERO), lands: true, expiry: Expiry::EndOfTurn, ..PlayGrant::CAST },
    );
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        // Mana abilities first.
        AbilityDef::Activated(ActivatedDef {
            text: "Tap an untapped artifact you control: Add {U}.",
            cost: Cost::tap_untapped(1, untapped_artifact_you_control),
            mana: Some(add_u),
            adds: ManaPool([0, 1, 0, 0, 0, 0]),
            ..ActivatedDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "When Urza enters, create a 0/0 colorless Construct artifact creature token with \"This token gets +1/+1 for each artifact you control.\"",
            listens: EventMask::ZONE_CHANGE,
            matches: this_enters,
            effect: make_construct,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "{5}: Shuffle your library, then exile the top card. Until end of turn, you may play that card without paying its mana cost.",
            cost: Cost { mana: mana(5, 0, 0, 0, 0, 0), ..Cost::NONE },
            effect: play_top,
            ..ActivatedDef::BLANK
        }),
    ],
    ..oracle::URZA_LORD_HIGH_ARTIFICER
};

pub static EXTRA: &[&CardDef] = &[&CONSTRUCT];
