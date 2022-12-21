-- on MYSQL 5.5 and below a trigger is needed to set the default timestamp to date_created field
CREATE TRIGGER on_before_insert
    BEFORE INSERT
    ON coupon
    FOR EACH ROW
    SET new.date_created = NOW();    