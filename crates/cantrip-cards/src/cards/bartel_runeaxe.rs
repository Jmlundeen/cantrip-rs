use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Bartel Runeaxe can't be the target of Aura spells." (CR 604.1).
fn bartel_runeaxe_cant_be_target_rule_on(
    q: &dyn LayerQuery,
    source: ObjectId,
    subject: ObjectId,
) -> bool {
    let asker = &Asker::new(source, q.controller(source));
    subject == asker.source
}

/// "Bartel Runeaxe can't be the target of Aura spells." (CR 604.1).
fn bartel_runeaxe_cant_be_target_rule_by(
    q: &dyn LayerQuery,
    source: ObjectId,
    by: &Asker,
    by_source: &Chars,
) -> bool {
    let asker = &Asker::new(source, q.controller(source));
    by.spell && (*by_source).has_subtype(q.words(asker.source).subtype(Subtype::Aura))
}

pub static CARD: CardDef = CardDef {
    targeting_rules: &[
        TargetingRule {
            kind: TargetingRuleKind::Forbid(bartel_runeaxe_cant_be_target_rule_by),
            applies: bartel_runeaxe_cant_be_target_rule_on,
        },
    ],
    ..oracle::BARTEL_RUNEAXE
};
