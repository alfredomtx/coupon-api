CREATE TABLE user (
  id int(11) NOT NULL AUTO_INCREMENT,
  email varchar(255) NOT NULL UNIQUE,
  password varchar(255) NOT NULL,
  role varchar(255) NOT NULL,
  PRIMARY KEY (id)
) ENGINE=InnoDB CHARSET=utf8 CHARSET=utf8 COLLATE=utf8_unicode_ci
