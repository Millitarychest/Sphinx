CREATE TABLE `ideas` (
  `id` int(11) NOT NULL AUTO_INCREMENT,
  `title` varchar(50) NOT NULL COMMENT 'idea title',
  `description` varchar(250) NOT NULL COMMENT 'short description',
  `lang` varchar(20) NOT NULL COMMENT 'recommended language',
  PRIMARY KEY (`id`)
);