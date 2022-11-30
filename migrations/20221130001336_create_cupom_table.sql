CREATE TABLE cupom (
  id int(11) NOT NULL AUTO_INCREMENT,
  code varchar(255) NOT NULL UNIQUE,
  discount int(11) NOT NULL,
  max_usage_count int(11) NULL,
  expiration_date DATETIME NULL,
  date_created TIMESTAMP NULL DEFAULT NULL,
  date_updated TIMESTAMP NULL DEFAULT NULL ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (id)
) ENGINE=InnoDB CHARSET=utf8 CHARSET=utf8 COLLATE=utf8_unicode_ci
