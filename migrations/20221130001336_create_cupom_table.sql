CREATE TABLE coupon (
  id int(11) NOT NULL AUTO_INCREMENT,
  code varchar(255) NOT NULL UNIQUE,
  discount int(11) NOT NULL,
  max_usage_count int(11) NULL, -- not actually being used currently, we will also need a new field to track the `current usage` count for the coupon
  expiration_date DATETIME NULL,
  active BOOLEAN NOT NULL DEFAULT 1,
  date_created DATETIME NOT NULL,
  date_updated TIMESTAMP NULL DEFAULT NULL ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (id)
) ENGINE=InnoDB CHARSET=utf8 CHARSET=utf8 COLLATE=utf8_unicode_ci
