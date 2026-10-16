use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_mechanics::{triggers::THIS_ENTERS, tokens};
use cantrip_oracle as oracle;

/// The 1/1 colorless Hero creature token job select makes.
static HERO: CardDef = CardDef {
    name: "Hero",
    has_cost: false,
    types: TypeSet::of(&[CardType::Creature]),
    subtypes: SubtypeSet::of(&[Subtype::Hero]),
    power: Some(1),
    toughness: Some(1),
    ..CardDef::BLANK
};

/// Job select: "create a 1/1 colorless Hero creature token, then attach this to it." An Equipment that has left the
/// battlefield attaches to nothing, and a token a replacement stopped is not there to be equipped (CR 701.3b).
fn job_select(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let made = tokens::create(g, item.controller, &HERO, 1)?;
    if let Some(&token) = made.first()
        && is_live(g, token)
        && g.is_on_battlefield(token)
        && is_live(g, item.object)
        && g.is_on_battlefield(item.object)
    {
        g.attach(item.object, token);
    }
    Ok(())
}

/// "Equipped creature is a Wizard in addition to its other types" (layer 4).
fn wizard(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    chars.subtypes = chars.subtypes.with(view.words(source).subtype(Subtype::Wizard));
}

/// The equipped creature has the Equipment's delayed ability 0 (CR 613.1f): one triggered ability with two conditions, an
/// ability it controls and that is gone the moment the Equipment stops being attached to it.
fn grant_growth(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let o = view.state().obj(source);
    let card = o.copy.map(|c| c.card).unwrap_or(o.card);
    if let Some(slot) = chars.abilities.iter().position(|a| a.is_none()) {
        chars.abilities[slot] = Some(AbilityRef::delayed(card, 0));
    }
}

/// "Whenever you cast a noncreature spell and whenever you draw your third card each turn" (CR 603.2, 121.2): the
/// history as it was just before the draw holds the two earlier ones.
fn cast_noncreature_or_draw_third(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::SpellCast { .. } => cantrip_mechanics::triggers::noncreature_spell_you_cast(q, me, event),
        Event::Drew { player, .. } if *player == me.controller && q.history().drawn(*player) == 2 => Fired::PLAIN,
        _ => None,
    }
}

fn grow(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if is_live(g, item.object) && g.is_on_battlefield(item.object) {
        g.add_counters(item.object, CounterKind::PlusOne, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, job_select, "Job select (When this Equipment enters, create a 1/1 colorless Hero creature token, then attach this to it.)"),
        equip(mana("{2}"), "Diana \u{2014} Equip {2}"),
    ],
    delayed: &[triggered(when_asked(EventMask(EventMask::SPELL_CAST.0 | EventMask::DREW.0), cast_noncreature_or_draw_third), grow, "Whenever you cast a noncreature spell and whenever you draw your third card each turn, put a +1/+1 counter on this creature.")],
    statics: &[
        static_ability(Layer::Type, attached_object, wizard, "Equipped creature is a Wizard in addition to its other types."),
        static_ability(Layer::Ability, attached_object, grant_growth, "Equipped creature has \"Whenever you cast a noncreature spell and whenever you draw your third card each turn, put a +1/+1 counter on this creature.\""),
    ],
    ..oracle::ASTROLOGIANS_PLANISPHERE
};

pub static EXTRA: &[&CardDef] = &[&HERO];
