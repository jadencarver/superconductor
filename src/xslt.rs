use std::process::Command;
use XML;

pub fn panel_xslt() -> String {
    let scss = Command::new("/usr/local/bin/sassc").arg("assets/__pm.scss").output().unwrap();
    let css = String::from_utf8(scss.stdout).unwrap();
    
    let markup = html! {
        xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" {
            xsl:output method="html" indent="yes" {}
            xsl:template match="/state/changes/change" {
                li {
                    xsl:element name="input" {
                        xsl:attribute name="id" {
                            xsl:value-of select="concat('__pm__changes_', path)" {}
                        }
                        xsl:attribute name="type" "checkbox"
                        xsl:if test="included='true'" {
                            xsl:attribute name="checked" {}
                        }
                        xsl:if test="removal='true'" {
                            xsl:attribute name="class" { "delete" }
                        }
                    }
                    label for="__pm__changes_{path}" {
                        xsl:value-of select="path" {}
                    }
                    button.button--tiny { " +10 -10" }
                }
            }
            xsl:template match="/" {
                div#__pm__panel {
                    style type="text/css" (css)
                    form#__pm__commit {
                        ul#__pm__commits {
                            li {
                                img src="http://en.gravatar.com/userimage/12799253/b889c035ec76c57ce679d12cbe01f2f4.png?s=24" {}
                                ul.properties {
                                    li {
                                        span.name "Status"
                                        span.before "In Progress"
                                        span.after  "Blocked"
                                    }
                                    li {
                                        span.name "Estimate"
                                        span.before "3"
                                        span.after  "5"
                                    }
                                }
                            }
                            li {
                                img src="http://en.gravatar.com/userimage/12799253/b889c035ec76c57ce679d12cbe01f2f4.png?s=24" {}
                                ul.properties {
                                    li {
                                        span.name "Status"
                                        span.before "In Progress"
                                        span.after  "Blocked"
                                    }
                                    li {
                                        span.name "Estimate"
                                        span.before "3"
                                        span.after  "5"
                                    }
                                }
                            }
                            li {
                                img src="http://en.gravatar.com/userimage/12799253/b889c035ec76c57ce679d12cbe01f2f4.png?s=24" {}
                                ul.properties {
                                    li {
                                        span.name "Status"
                                        span.before "In Progress"
                                        span.after  "Blocked"
                                    }
                                    li {
                                        span.name "Estimate"
                                        span.before "3"
                                        span.after  "5"
                                    }
                                }
                            }
                            li {
                                img src="http://en.gravatar.com/userimage/12799253/b889c035ec76c57ce679d12cbe01f2f4.png?s=24" {}
                                button.attachments {
                                    svg id="i-paperclip" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32" width="24" height="24" fill="none" stroke="currentcolor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" {
                                        path d="M10 9 L10 24 C10 28 13 30 16 30 19 30 22 28 22 24 L22 6 C22 3 20 2 18 2 16 2 14 3 14 6 L14 23 C14 24 15 25 16 25 17 25 18 24 18 23 L18 9" {}
                                    }
                                }
                                blockquote { "Got it working!" }
                                ul.properties {
                                    li {
                                        span.name "Status"
                                        span.before "Blocked"
                                        span.after  "Finished"
                                    }
                                }
                            }
                        }
                        textarea id="__pm__commit__message" name="message" placeholder="Enter your message" {}
                        input type="submit" value="Save Update" {}
                        details#__pm__commit__changes {
                            summary { "Include Changes" }
                            ul {
                                xsl:apply-templates select="/state/changes/change" {}
                            }
                        }
                    }
                    header {
                        xsl:value-of select="/state/user/name" {}
                    }
                    ul.tiles {
                        li draggable="true" {
                            strong {
                                xsl:value-of select="/state/effort" {}
                            }
                        }
                    }
                }
            }
        }
    };
    format!("{}{}", XML, markup.into_string())
}


