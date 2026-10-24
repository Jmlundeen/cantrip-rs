use cantrip_api::*;
use cantrip_mechanics::{hand, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "When this enchantment enters, draw a card, then you may put a land card from your hand onto the battlefield. If you put a
/// Cave onto the battlefield this way, you gain 4 life." (CR 608.2c): the draw ends the sentence before the land is chosen from
/// the hand it left. The life is gained only if the land chosen arrived on the battlefield and is a Cave there (CR 205.3i).
/// "Cave" is a land type word of its text, read through its words (CR 612.2).
fn draw_put_a_land(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    g.draw(you, 1);
    g.end_sentence();
    if let Some(land) = hand::may_put_onto_battlefield(g, you, item.object, |c| c.is(CardType::Land))?
        && g.obj(land).zone.is(ZoneKind::Battlefield)
        && g.chars(land).has_subtype(g.words(item.object).subtype(Subtype::Cave))
    {
        g.gain_life(you, 4);
    }
    Ok(())
}

/// "Lands you control enter untapped." (CR 614.1d): every land entering under your control, judged as it would exist on the
/// battlefield (CR 614.12), whether it would enter tapped as the event stands or not. So a land put onto the battlefield
/// tapped enters untapped, and a land whose own "enters tapped" also applies has its controller order the two (CR 616.1):
/// its own first, then this, and it enters untapped; this first, then its own, and it enters tapped (the rulings).
fn lands_you_control_enter(q: &dyn Query, asker: &Asker, proposal: &Proposal) -> bool {
    let Proposal::ZoneChange(z) = proposal else { return false };
    z.to.is(ZoneKind::Battlefield) && z.controller == asker.controller && q.chars(z.object).is(CardType::Land)
}

fn untapped(_: &dyn Query, _: &Asker, proposal: &mut Proposal) {
    if let Proposal::ZoneChange(z) = proposal {
        z.tapped = false;
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ENTERS,
        draw_put_a_land,
        "When this enchantment enters, draw a card, then you may put a land card from your hand onto the battlefield. If you put a Cave onto the battlefield this way, you gain 4 life.",
    )],
    replacements: &[replacement(ReplacementKind::ZoneChange, lands_you_control_enter, untapped, "lands you control enter untapped")],
    ..oracle::SPELUNKING
};
