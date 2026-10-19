use cantrip_api::*;
use cantrip_mechanics::triggers::BEGINNING_OF_YOUR_COMBAT;
use cantrip_mechanics::{targets, tokens};
use cantrip_oracle as oracle;

/// "Whenever another Rat you control enters" (CR 603.6a): a permanent other than this one, under your control as it
/// entered, that is a Rat as it enters (a creature type read through this card's words, CR 612). The entering Rat is the
/// trigger's subject; Skitter's own Rat tokens count, and so does a Rat that is not a creature.
fn another_rat_you_control_enters(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { new, to, .. }
            if *new != me.source
                && to.is(ZoneKind::Battlefield)
                && q.controller(*new) == me.controller
                && q.chars(*new).has_subtype(q.words(me.source).subtype(Subtype::Rat)) =>
        {
            Fired::about(Target::Object(*new))
        }
        _ => None,
    }
}

/// "Exile up to one target card from an opponent's graveyard."
fn exile_card(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(card) = targets::object(g, item, 0) {
        g.exile_all(&[card]);
    }
    Ok(())
}

/// "Create a 1/1 black Rat creature token with 'This token can't block.'" (CR 111.1, 509.1b)
fn make_rat(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::RAT_CANT_BLOCK, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            when_asked(EventMask::ZONE_CHANGE, another_rat_you_control_enters),
            exile_card,
            "Whenever another Rat you control enters, exile up to one target card from an opponent's graveyard.",
        )
        .targets(&[targets::CARD_IN_AN_OPPONENTS_GRAVEYARD.optional()]),
        triggered(
            BEGINNING_OF_YOUR_COMBAT,
            make_rat,
            "At the beginning of combat on your turn, create a 1/1 black Rat creature token with \"This token can't block.\"",
        ),
    ],
    ..oracle::LORD_SKITTER_SEWER_KING
};

pub static EXTRA: &[&CardDef] = &[&tokens::RAT_CANT_BLOCK];
