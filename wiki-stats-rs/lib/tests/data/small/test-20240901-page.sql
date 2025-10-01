-- MariaDB dump 10.19  Distrib 10.5.23-MariaDB, for debian-linux-gnu (x86_64)
--
-- Host: db1166    Database: loginwiki
-- ------------------------------------------------------
-- Server version	10.6.17-MariaDB-log

/*!40101 SET @OLD_CHARACTER_SET_CLIENT=@@CHARACTER_SET_CLIENT */;
/*!40101 SET @OLD_CHARACTER_SET_RESULTS=@@CHARACTER_SET_RESULTS */;
/*!40101 SET @OLD_COLLATION_CONNECTION=@@COLLATION_CONNECTION */;
/*!40101 SET NAMES utf8mb4 */;
/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;

--
-- Table structure for table `page`
--

DROP TABLE IF EXISTS `page`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `page` (
  `page_id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `page_namespace` int(11) NOT NULL,
  `page_title` varbinary(255) NOT NULL,
  `page_is_redirect` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `page_is_new` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `page_random` double unsigned NOT NULL,
  `page_touched` binary(14) NOT NULL,
  `page_links_updated` binary(14) DEFAULT NULL,
  `page_latest` int(10) unsigned NOT NULL,
  `page_len` int(10) unsigned NOT NULL,
  `page_content_model` varbinary(32) DEFAULT NULL,
  `page_lang` varbinary(35) DEFAULT NULL,
  PRIMARY KEY (`page_id`),
  UNIQUE KEY `page_name_title` (`page_namespace`,`page_title`),
  KEY `page_random` (`page_random`),
  KEY `page_len` (`page_len`),
  KEY `page_redirect_namespace_len` (`page_is_redirect`,`page_namespace`,`page_len`)
) ENGINE=InnoDB AUTO_INCREMENT=23 DEFAULT CHARSET=binary;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `page`
--

/*!40000 ALTER TABLE `page` DISABLE KEYS */;
INSERT INTO `page` VALUES (1,0,'Main_Page',0,0,0.78387112572,'20230603201208','20230603201208',30,678,'wikitext',NULL),(2,8,'Sp-contributions-footer',0,0,0.160279553485,'20230603011507','20230603011506',22,284,'wikitext',NULL),(3,8,'Checkuser-userlinks',0,0,0.809690799557,'20230603011507','20230603011506',23,42,'wikitext',NULL),(5,8,'Checkuser-userlinks-ip',0,0,0.697152593104,'20230604134318','20230604134318',33,307,'wikitext',NULL),(7,2,'MarcoAurelio',0,1,0.138018493958,'20230603011507','20230603011506',20,7,'wikitext',NULL),(8,8,'Checkuser-toollinks',0,0,0.505053750651,'20230603235451','20230603235451',53,825,'wikitext',NULL),(10,8,'Noarticletext-nopermission',0,0,0.961690175459,'20230604134318','20230604134318',39,587,'wikitext',NULL),(11,8,'Noarticletext',0,0,0.441713411451,'20230604134318','20230604134318',37,596,'wikitext',NULL),(12,2,'Operator873/common.js',0,0,0.10194505541,'20230604210118','20230604210118',45,553,'javascript',NULL),(15,2,'TheresNoTime/Sandbox',0,1,0.573194194397,'20230603235451','20230603235451',50,1,'wikitext',NULL),(16,8,'Sp-contributions-footer-anon',0,1,0.841870477329,'20230603235451','20230603235451',51,512,'wikitext',NULL),(17,8,'Sp-contributions-footer-anon-range',0,1,0.091451841397,'20230603235451','20230603235451',52,516,'wikitext',NULL),(20,4,'General_disclaimer',1,1,0.966582563716,'20231112031621','20231112031615',56,50,'wikitext',NULL),(21,4,'About',1,1,0.71492878793,'20231112031739','20231112031736',57,23,'wikitext',NULL);
/*!40000 ALTER TABLE `page` ENABLE KEYS */;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40101 SET CHARACTER_SET_CLIENT=@OLD_CHARACTER_SET_CLIENT */;
/*!40101 SET CHARACTER_SET_RESULTS=@OLD_CHARACTER_SET_RESULTS */;
/*!40101 SET COLLATION_CONNECTION=@OLD_COLLATION_CONNECTION */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

-- Dump completed on 2024-09-01 10:42:25
