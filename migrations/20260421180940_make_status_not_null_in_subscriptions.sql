-- SQL transaction for the migration
-- we set all the previous subscriptions as confirmed and then make the column NOT NULL
BEGIN;
    UPDATE subscriptions
        SET status = 'confirmed'
        WHERE status IS NULL;
    ALTER TABLE subscriptions ALTER COLUMN status SET NOT NULL;
COMMIT;
