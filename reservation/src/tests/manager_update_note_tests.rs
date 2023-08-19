#[cfg(test)]
mod tests {
    use crate::{tests::test_utils::*, Rsvp};

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn update_note_should_work() {
        let (rsvp, manager) = make_reservation_with_yang_template(migrated_pool.clone()).await;
        let rsvp = manager.update_note(
            rsvp.id,
            "I spent all of my money so plz gives me a wonderful feeling. I want to have a wonderful experience.".to_string(),
        ).await.unwrap();

        assert_eq!(rsvp.note, "I spent all of my money so plz gives me a wonderful feeling. I want to have a wonderful experience.");
    }
}
