fn learn_dialogue_topic(
    ctx: &ReducerContext,
    character_id: u64,
    session: &DialogueSession,
    topic: &str,
) {
    let id = format!("{character_id}:{}:{topic}", session.conversation_id);
    if ctx.db.character_topic_knowledge().id().find(&id).is_none() {
        ctx.db
            .character_topic_knowledge()
            .insert(CharacterTopicKnowledge {
                id,
                character_id,
                conversation_id: session.conversation_id.clone(),
                topic_id: topic.to_owned(),
                learned_micros: ctx.timestamp.to_micros_since_unix_epoch(),
            });
    }
}
