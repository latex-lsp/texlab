/*
 * marc_auth.c
 *
 * Copyright (c) Chris Putnam 2004-2019
 *
 * Source code released under the GPL version 2
 *
 */
#include "marc_auth.h"
#include <string.h>

static const char *marc_genre[] = {
	"abstract or summary",
	"art original",
	"art reproduction",
	"article",
	"atlas",
	"autobiography",
	"bibliography",
	"biography",
	"book",
	"calendar",
	"catalog",
	"chart",
	"comic or graphic novel",
	"comic strip",
	"conference publication",
	"database",
	"dictionary",
	"diorama",
	"directory",
	"discography",
	"drama",
	"encyclopedia",
	"essay",
	"festschrift",
	"fiction",
	"filmography",
	"filmstrip",
	"finding aid",
	"flash card",
	"folktale",
	"font",
	"game",
	"government publication",
	"graphic",
	"globe",
	"handbook",
	"history",
	"humor, satire",
	"hymnal",
	"index",
	"instruction",
	"interview",
	"issue",
	"journal",
	"kit",
	"language instruction",
	"law report or digest",
	"legal article",
	"legal case and case notes",
	"legislation",
	"letter",
	"loose-leaf",
	"map",
	"memoir",
	"microscope slide",
	"model",
	"motion picture",
	"multivolume monograph",
	"newspaper",
	"novel",
	"numeric data",
	"offprint",
	"online system or service",
	"patent",
	"periodical",
	"picture",
	"poetry",
	"programmed text",
	"realia",
	"rehearsal",
	"remote sensing image",
	"reporting",
	"review",
	"series",
	"short story",
	"slide",
	"sound",
	"speech",
	"standard or specification",
	"statistics",
	"survey of literature",
	"technical drawing",
	"technical report",
	"thesis",
	"toy",
	"transparency",
	"treaty",
	"videorecording",
	"web site",
	"yearbook",
};
static const int nmarc_genre = sizeof( marc_genre ) / sizeof( const char* );

static const char *marc_resource[] = {
	"cartographic",
	"kit",
	"mixed material",
	"moving image",
	"notated music",
	"software, multimedia",
	"sound recording",
	"sound recording - musical",
	"sound recording - nonmusical",
	"still image",
	"text",
	"three dimensional object"
};
static const int nmarc_resource = sizeof( marc_resource ) / sizeof( const char* );


/* www.loc.gov/marc/relators/relacode.html */

typedef struct marc_trans {
	char *abbreviation;
	char *internal_name;
} marc_trans;

static const marc_trans relators[] = {
	{ "abr",	"ABRIDGER"                     },	/* Abridger */
	{ "acp",	"ART_COPYIST"                  },	/* Art copyist */
	{ "act",	"ACTOR"                        },	/* Actor */
	{ "adi", 	"ART_DIRECTOR"                 },	/* Art director */
	{ "adp",	"ADAPTER"                      },	/* Adapter */
	{ "aft",	"AUTHOR"                       },	/* Author of afterword, colophon, etc. */
	{ "anl",	"ANALYST"                      }, 	/* Analyst */
	{ "anm",	"ANIMATOR"                     },	/* Animator */
	{ "ann",	"ANNOTATOR"                    },	/* Annotator */
	{ "ant",	"BIBLIOGRAPHIC_ANTECENDENT"    },	/* Bibliographic antecedent */
	{ "ape",	"APPELLEE"                     },	/* Appellee */
	{ "apl",	"APPELLANT"                    }, 	/* Appellant */
	{ "app",	"APPLICANT"                    },	/* Applicant */
	{ "aqt",	"AUTHOR"                       },	/* Author in quotations or text abstracts */
	{ "arc",	"ARCHITECT"                    },	/* Architect */
	{ "ard",	"ARTISTIC_DIRECTOR"            },	/* Artistic director */
	{ "arr",	"ARRANGER"                     },	/* Arranger */
	{ "art",	"ARTIST"                       },	/* Artist */
	{ "asg",	"ASSIGNEE"                     },	/* Assignee */
	{ "asn",	"ASSOCIATED_NAME"              },	/* Associated name */
	{ "ato",	"AUTOGRAPHER"                  },	/* Autographer */
	{ "att",	"ATTRIBUTED_NAME"              },	/* Attributed name */
	{ "auc",	"AUCTIONEER"                   },	/* Auctioneer */
	{ "aud",	"AUTHOR"                       },	/* Author of dialog */
	{ "aui",	"AUTHOR"                       },	/* Author of introduction, etc. */
	{ "aus",	"AUTHOR"                       },	/* Screenwriter */
	{ "aut",	"AUTHOR"                       },	/* Author */
	{ "author",     "AUTHOR"                       },
	{ "bdd",	"BINDING_DESIGNER"             },	/* Binding designer */
	{ "bjd",	"BOOKJACKET_DESIGNER"          },	/* Bookjacket designer */
	{ "bkd",	"BOOK_DESIGNER"                },	/* Book designer */
	{ "bkp",	"BOOK_PRODUCER"                },	/* Book producer */
	{ "blw",	"AUTHOR"                       },	/* Blurb writer */
	{ "bnd",	"BINDER"                       },	/* Binder */
	{ "bpd",	"BOOKPLATE_DESIGNER"           },	/* Bookplate designer */
	{ "brd",	"BROADCASTER"                  },	/* Broadcaster */
	{ "brl",	"BRAILLE_EMBOSSER"             },	/* Braille embosser */
	{ "bsl",	"BOOKSELLER"                   },	/* Bookseller */
	{ "cas",	"CASTER"                       },	/* Caster */
	{ "ccp",	"CONCEPTOR"                    },	/* Conceptor */
	{ "chr",	"CHOREOGRAPHER"                },	/* Choreographer */
	{ "clb",	"COLLABORATOR"                 },	/* Collaborator */
	{ "cli",	"CLIENT"                       },	/* Client */
	{ "cll",	"CALLIGRAPHER"                 },	/* Calligrapher */
	{ "clr",	"COLORIST"                     },	/* Colorist */
	{ "clt",	"COLLOTYPER"                   },	/* Collotyper */
	{ "cmm",	"COMMENTATOR"                  },	/* Commentator */
	{ "cmp",	"COMPOSER"                     },	/* Composer */
	{ "cmt",	"COMPOSITOR"                   },	/* Compositor */
	{ "cnd",	"CONDUCTOR"                    },	/* Conductor */
	{ "cng",	"CINEMATOGRAPHER"              },	/* Cinematographer */
	{ "cns",	"CENSOR"                       },	/* Censor */
	{ "coe",	"CONTESTANT-APPELLEE"          },	/* Contestant-appellee */
	{ "col",	"COLLECTOR"                    },	/* Collector */
	{ "com",	"COMPILER"                     },	/* Compiler */
	{ "con",	"CONSERVATOR"                  },	/* Conservator */
	{ "cor",	"COLLECTION_REGISTRAR"         },	/* Collection registrar */
	{ "cos",	"CONTESTANT"                   },	/* Contestant */
	{ "cot",	"CONTESTANT-APPELLANT"         },	/* Contestant-appellant */
	{ "cou",	"COURT_GOVERNED"               },	/* Court governed */
	{ "cov",	"COVER_DESIGNER"               },	/* Cover designer */
	{ "cpc",	"COPYRIGHT_CLAIMANT"           },	/* Copyright claimant */
	{ "cpe",	"COMPLAINANT-APPELLEE"         },	/* Complainant-appellee */
	{ "cph",	"COPYRIGHT_HOLDER"             },	/* Copyright holder */
	{ "cpl",	"COMPLAINANT"                  },	/* Complainant */
	{ "cpt",	"COMPLAINANT-APPELLANT"        },	/* Complainant-appellant */
	{ "cre",	"AUTHOR"                       },	/* Creator */
	{ "creator",    "AUTHOR"                       },
	{ "crp",	"CORRESPONDENT"                },	/* Correspondent */
	{ "crr",	"CORRECTOR"                    },	/* Corrector */
	{ "crt",	"COURT_REPORTER"               },	/* Court reporter */
	{ "csl",	"CONSULTANT"                   },	/* Consultant */
	{ "csp",	"CONSULTANT_TO_A_PROJECT"      },	/* Consultant to a project */
	{ "cst",	"COSTUME_DESIGNER"             },	/* Costume designer */
	{ "ctb",	"CONTRIBUTOR"                  },	/* Contributor */
	{ "cte",	"CONTESTEE-APPELLEE"           },	/* Contestee-appellee */
	{ "ctg",	"CARTOGRAPHER"                 },	/* Cartographer */
	{ "ctr",	"CONTRACTOR"                   },	/* Contractor */
	{ "cts",	"CONTESTEE"                    },	/* Contestee */
	{ "ctt",	"CONTESTEE-APPELLANT"          },	/* Contestee-appellant */
	{ "cur",	"CURATOR"                      },	/* Curator */
	{ "cwt",	"COMMENTATOR_FOR_WRITTEN_TEXT" },	/* Commentator for written text */
	{ "dbp",	"DISTRIBUTION_PLACE"           },	/* Distribution place */
	{ "degree grantor", "DEGREEGRANTOR"            },
	{ "dfd",	"DEFENDANT"                    },	/* Defendant */
	{ "dfe",	"DEFENDANT-APPELLEE"           },	/* Defendant-appellee */
	{ "dft",	"DEFENDANT-APPELLANT"          },	/* Defendant-appellant */
	{ "dgg",	"DEGREEGRANTOR"                },	/* Degree granting institution */
	{ "dgs",	"DEGREE_SUPERVISOR"            },	/* Degree supervisor */
	{ "dis",	"DISSERTANT"                   },	/* Dissertant */
	{ "dln",	"DELINEATOR"                   },	/* Delineator */
	{ "dnc",	"DANCER"                       },	/* Dancer */
	{ "dnr",	"DONOR"                        },	/* Donor */
	{ "dpc",	"DEPICTED"                     },	/* Depicted */
	{ "dpt",	"DEPOSITOR"                    },	/* Depositor */
	{ "drm",	"DRAFTSMAN"                    }, 	/* Draftsman */
	{ "drt",	"DIRECTOR"                     }, 	/* Director */
	{ "dsr",	"DESIGNER"                     }, 	/* Designer */
	{ "dst",	"DISTRIBUTOR"                  }, 	/* Distributor */
	{ "dtc",	"DATA_CONTRIBUTOR"             }, 	/* Data contributor */
	{ "dte",	"DEDICATEE"                    }, 	/* Dedicatee */
	{ "dtm",	"DATA_MANAGER"                 }, 	/* Data manager */
	{ "dto",	"DEDICATOR"                    }, 	/* Dedicator */
	{ "dub",	"AUTHOR"                       },	/* Dubious author */
	{ "edc",	"EDITOR"                       }, 	/* Editor of compilation */
	{ "edm",	"EDITOR"                       },	/* Editor of moving image work */
	{ "edt",	"EDITOR"                       },	/* Editor */
	{ "egr",	"ENGRAVER"                     }, 	/* Engraver */
	{ "elg",	"ELECTRICIAN" 	               },	/* Electrician */
	{ "elt",	"ELECTROTYPER"                 },	/* Electrotyper */
	{ "eng",	"ENGINEER"                     }, 	/* Engineer */
	{ "enj",	"ENACTING_JURISICTION"         }, 	/* Enacting jurisdiction */
	{ "etr",	"ETCHER"                       }, 	/* Etcher */
	{ "evp",	"EVENT_PLACE"                  }, 	/* Event place */
	{ "exp",	"EXPERT"                       }, 	/* Expert */
	{ "fac",	"FACSIMILIST"                  }, 	/* Facsimilist */
	{ "fds",	"FILM_DISTRIBUTOR"             }, 	/* Film distributor */
	{ "fld",	"FIELD_DIRECTOR"               }, 	/* Field director */
	{ "flm",	"FILM_EDITOR"                  }, 	/* Film editor */
	{ "fmd",	"FILM_DIRECTOR"                }, 	/* Film director */
	{ "fmk",	"FILMMAKER"                    }, 	/* Filmmaker */
	{ "fmo",	"FORMER_OWNER"                 }, 	/* Former owner */
	{ "fmp",	"FILM_PRODUCER"                }, 	/* Film producer */
	{ "fnd",	"FUNDER"                       }, 	/* Funder */
	{ "fpy",	"FIRST_PARTY"                  }, 	/* First party */
	{ "frg",	"FORGER"                       }, 	/* Forger */
	{ "gis",	"GEOGRAPHIC_INFORMATON_SPECIALIST" }, 	/* Geographic information specialist */
	{ "grt",	"GRAPHIC_TECHNICIAN"           }, 	/* Graphic technician */
	{ "his",	"HOST_INSTITUTION"             }, 	/* Host institution */
	{ "hnr",	"HONOREE"                      }, 	/* Honoree */
	{ "hst",	"HOST"                         }, 	/* Host */
	{ "ill",	"ILLUSTRATOR"                  }, 	/* Illustrator */
	{ "ilu",	"ILLUMINATOR"                  }, 	/* Illuminator */
	{ "ins",	"INSCRIBER"                    }, 	/* Inscriber */
	{ "inv",	"INVENTOR"                     }, 	/* Inventor */
	{ "isb",	"ISSUING_BODY"                 }, 	/* Issuing body */
	{ "itr",	"INSTRUMENTALIST"              }, 	/* Instrumentalist */
	{ "ive",	"INTERVIEWEE"                  }, 	/* Interviewee */
	{ "ivr",	"INTERVIEWER"                  }, 	/* Interviewer */
	{ "jud",	"JUDGE"                        }, 	/* Judge */
	{ "jug",	"JURISDICTION_GOVERNED"        }, 	/* Jurisdiction governed */
	{ "lbr",	"LABORATORY"                   }, 	/* Laboratory */
	{ "lbt",	"LIBRETTIST"                   }, 	/* Librettist */
	{ "ldr",	"LABORATORY_DIRECTORY"         }, 	/* Laboratory director */
	{ "led",	"LEAD"                         }, 	/* Lead */
	{ "lee",	"LIBELEE-APPELLEE"             }, 	/* Libelee-appellee */
	{ "lel",	"LIBELEE"                      }, 	/* Libelee */
	{ "len",	"LENDER"                       }, 	/* Lender */
	{ "let",	"LIBELEE-APPELLANT"            }, 	/* Libelee-appellant */
	{ "lgd",	"LIGHTING_DESIGNER"            }, 	/* Lighting designer */
	{ "lie",	"LIBELANT-APPELLEE"            }, 	/* Libelant-appellee */
	{ "lil",	"LIBELANT"                     }, 	/* Libelant */
	{ "lit",	"LIBELANT-APPELLANT"           }, 	/* Libelant-appellant */
	{ "lsa",	"LANDSCAPE_ARCHITECT"          }, 	/* Landscape architect */
	{ "lse",	"LICENSEE"                     }, 	/* Licensee */
	{ "lso",	"LICENSOR"                     }, 	/* Licensor */
	{ "ltg",	"LITHOGRAPHER"                 }, 	/* Lithographer */
	{ "lyr",	"LYRICIST"                     }, 	/* Lyricist */
	{ "mcp",	"MUSIC_COPYIST"                }, 	/* Music copyist */
	{ "mdc",	"METADATA_CONTACT"             }, 	/* Metadata contact */
	{ "med",	"MEDIUM"                       }, 	/* Medium */
	{ "mfp",	"MANUFACTURE_PLACE"            }, 	/* Manufacture place */
	{ "mfr",	"MANUFACTURER"                 }, 	/* Manufacturer */
	{ "mod",	"MODERATOR"                    }, 	/* Moderator */
	{ "mon",	"THESIS_EXAMINER"              }, 	/* Monitor */
	{ "mrb",	"MARBLER"                      }, 	/* Marbler */
	{ "mrk",	"EDITOR"                       }, 	/* Markup editor */
	{ "msd",	"MUSICAL_DIRECTOR"             }, 	/* Musical director */
	{ "mte",	"METAL-ENGRAVER"               }, 	/* Metal-engraver */
	{ "mtk",	"MINUTE_TAKER"                 },       /* Minute taker */
	{ "mus",	"MUSICIAN"                     }, 	/* Musician */
	{ "nrt",	"NARRATOR"                     }, 	/* Narrator */
	{ "opn",	"THESIS_OPPONENT"              }, 	/* Opponent */
	{ "org",	"ORIGINATOR"                   }, 	/* Originator */
	{ "organizer of meeting", "ORGANIZER"          },
	{ "orm",	"ORGANIZER"                    }, 	/* Organizer */
	{ "osp",	"ONSCREEN_PRESENTER"           }, 	/* Onscreen presenter */
	{ "oth",	"THESIS_OTHER"                 }, 	/* Other */
	{ "own",	"OWNER"                        }, 	/* Owner */
	{ "pan",	"PANELIST"                     }, 	/* Panelist */
	{ "pat",	"PATRON"                       }, 	/* Patron */
	{ "patent holder", "ASSIGNEE"                  },
	{ "pbd",	"PUBLISHING_DIRECTOR"          }, 	/* Publishing director */
	{ "pbl",	"PUBLISHER"                    }, 	/* Publisher */
	{ "pdr",	"PROJECT_DIRECTOR"             },	/* Project director */
	{ "pfr",	"PROOFREADER"                  }, 	/* Proofreader */
	{ "pht",	"PHOTOGRAPHER"                 }, 	/* Photographer */
	{ "plt",	"PLATEMAKER"                   }, 	/* Platemaker */
	{ "pma",	"PERMITTING_AGENCY"            }, 	/* Permitting agency */
	{ "pmn",	"PRODUCTION_MANAGER"           }, 	/* Production manager */
	{ "pop",	"PRINTER_OF_PLATES"            }, 	/* Printer of plates */
	{ "ppm",	"PAPERMAKER"                   }, 	/* Papermaker */
	{ "ppt",	"PUPPETEER"                    }, 	/* Puppeteer */
	{ "pra",	"PRAESES"                      }, 	/* Praeses */
	{ "prc",	"PROCESS_CONTRACT"             }, 	/* Process contact */
	{ "prd",	"PRODUCTION_PERSONNEL"         }, 	/* Production personnel */
	{ "pre",	"PRESENTER"                    },	/* Presenter */
	{ "prf",	"PERFORMER"                    }, 	/* Performer */
	{ "prg",	"AUTHOR"                       }, 	/* Programmer */
	{ "prm",	"PRINTMAKER"                   }, 	/* Printmaker */
	{ "prn",	"PRODUCTION_COMPANY"           }, 	/* Production company */
	{ "pro",	"PRODUCER"                     }, 	/* Producer */
	{ "prp",	"PRODUCTION_PLACE"             }, 	/* Production place */
	{ "prs",	"PRODUCTION_DESIGNER"          }, 	/* Production designer */
	{ "prt",	"PRINTER"                      }, 	/* Printer */
	{ "prv",	"PROVIDER"                     }, 	/* Provider */
	{ "pta",	"PATENT_APPLICANT"             }, 	/* Patent applicant */
	{ "pte",	"PLAINTIFF-APPELLEE"           }, 	/* Plaintiff-appellee */
	{ "ptf",	"PLAINTIFF"                    }, 	/* Plaintiff */
	{ "pth",	"ASSIGNEE"                     }, 	/* Patent holder */
	{ "ptt",	"PLAINTIFF-APPELLANT"          }, 	/* Plaintiff-appellant */
	{ "pup",	"PUBLICATION_PLACE"            }, 	/* Publication place */
	{ "rbr",	"RUBRICATOR"                   }, 	/* Rubricator */
	{ "rcd",	"RECORDIST"                    }, 	/* Recordist */
	{ "rce",	"RECORDING_ENGINEER"           }, 	/* Recording engineer */
	{ "rcp",	"ADDRESSEE"                    }, 	/* Addressee */
	{ "rdd",	"RADIO_DIRECTOR"               }, 	/* Radio director */
	{ "red",	"REDAKTOR"                     }, 	/* Redaktor */
	{ "ren",	"RENDERER"                     }, 	/* Renderer */
	{ "res",	"RESEARCHER"                   }, 	/* Researcher */
	{ "rev",	"REVIEWER"                     }, 	/* Reviewer */
	{ "rpc",	"RADIO_PRODUCER"               }, 	/* Radio producer */
	{ "rps",	"REPOSITORY"                   }, 	/* Repository */
	{ "rpt",	"REPORTER"                     }, 	/* Reporter */
	{ "rpy",	"RESPONSIBLE_PARTY"            }, 	/* Responsible party */
	{ "rse",	"RESPONDENT-APPELLEE"          }, 	/* Respondent-appellee */
	{ "rsg",	"RESTAGER"                     }, 	/* Restager */
	{ "rsp",	"RESPONDENT"                   }, 	/* Respondent */
	{ "rsr",	"RESTORATIONIST"               }, 	/* Restorationist */
	{ "rst",	"RESPONDENT-APPELLANT"         }, 	/* Respondent-appellant */
	{ "rth",	"RESEARCH_TEAM_HEAD"           }, 	/* Research team head */
	{ "rtm",	"RESEARCH_TEAM_MEMBER"         }, 	/* Research team member */
	{ "sad",	"SCIENTIFIC_ADVISOR"           }, 	/* Scientific advisor */
	{ "sce",	"SCENARIST"                    }, 	/* Scenarist */
	{ "scl",	"SCULPTOR"                     }, 	/* Sculptor */
	{ "scr",	"SCRIBE"                       }, 	/* Scribe */
	{ "sds",	"SOUND_DESIGNER"               }, 	/* Sound designer */
	{ "sec",	"SECRETARY"                    }, 	/* Secretary */
	{ "sgd",	"STAGE_DIRECTOR"               }, 	/* Stage director */
	{ "sgn",	"SIGNER"                       }, 	/* Signer */
	{ "sht",	"SUPPORTING_HOST"              }, 	/* Supporting host */
	{ "sll",	"SELLER"                       }, 	/* Seller */
	{ "sng",	"SINGER"                       }, 	/* Singer */
	{ "spk",	"SPEAKER"                      }, 	/* Speaker */
	{ "spn",	"SPONSOR"                      }, 	/* Sponsor */
	{ "spy",	"SECOND_PARTY"                 }, 	/* Second party */
	{ "srv",	"SURVEYOR"                     }, 	/* Surveyor */
	{ "std",	"SET_DESIGNER"                 },	/* Set designer */
	{ "stg",	"SETTING"                      },	/* Setting */
	{ "stl",	"STORYTELLER"                  },	/* Storyteller */
	{ "stm",	"STAGE_MANAGER"                },	/* Stage manager */
	{ "stn",	"STANDARDS_BODY"               },	/* Standards body */
	{ "str",	"STEREOTYPER"                  },	/* Stereotyper */
	{ "tcd",	"TECHNICAL_DIRECTOR"           },	/* Technical director */
	{ "tch",	"TEACHER"                      },	/* Teacher */
	{ "ths",	"THESIS_ADVISOR"               },	/* Thesis advisor */
	{ "tld",	"TELEVISION_DIRECTOR"          },	/* Television director */
	{ "tlp",	"TELEVISION_PRODUCER"          },	/* Television producer */
	{ "trc",	"TRANSCRIBER"                  },	/* Transcriber */
	{ "trl",	"TRANSLATOR"                   },	/* Translator */
	{ "tyd",	"TYPE_DIRECTOR"                }, 	/* Type designer */
	{ "tyg",	"TYPOGRAPHER"                  }, 	/* Typographer */
	{ "uvp",	"UNIVERSITY_PLACE"             }, 	/* University place */
	{ "vac",	"VOICE_ACTOR"                  }, 	/* Voice actor */
	{ "vdg",	"VIDEOGRAPHER"                 }, 	/* Videographer */
	{ "voc",	"VOCALIST"                     }, 	/* Vocalist */
	{ "wac",	"AUTHOR"                       }, 	/* Writer of added commentary */
	{ "wal",	"AUTHOR"                       }, 	/* Writer of added lyrics */
	{ "wam",	"AUTHOR"                       }, 	/* Writer of accompanying material */
	{ "wat",	"AUTHOR"                       }, 	/* Writer of added text */
	{ "wdc",	"WOODCUTTER"                   }, 	/* Woodcutter */
	{ "wde",	"WOOD_ENGRAVER"                }, 	/* Wood engraver */
	{ "win",	"AUTHOR"                       }, 	/* Writer of introduction */
	{ "wit",	"WITNESS"                      }, 	/* Witness */
	{ "wpr",	"AUTHOR"                       }, 	/* Writer of preface */
	{ "wst",	"AUTHOR"                       }, 	/* Writer of supplementary textual content */
};

static const int nrealtors = sizeof( relators ) / sizeof( relators[0] );

char *
marc_convertrole( const char *query )
{
	int i;

	for ( i=0; i<nrealtors; ++i ) {
		if ( !strcasecmp( query, relators[i].abbreviation ) )
			return relators[i].internal_name;
	}
	return NULL;
}

static int
position_in_list( const char *list[], int nlist, const char *query )
{
	int i;
	for ( i=0; i<nlist; ++i ) {
		if ( !strcasecmp( query, list[i] ) ) return i;
	}
	return -1;
}

int
marc_findgenre( const char *query )
{
	return position_in_list( marc_genre, nmarc_genre, query );
}

int
is_marc_genre( const char *query )
{
	if ( marc_findgenre( query ) != -1 ) return 1;
	else return 0;
}

int
marc_findresource( const char *query )
{
	return position_in_list( marc_resource, nmarc_resource, query );
}

int
is_marc_resource( const char *query )
{
	if ( marc_findresource( query ) != -1 ) return 1;
	else return 0;
}
