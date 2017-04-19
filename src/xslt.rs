use std::process::Command;
use XML;

pub fn panel_xslt() -> String {
    let scss = Command::new("/usr/local/bin/sassc").arg("/Users/jadencarver/dev/superconductor/assets/__pm.scss").output().unwrap();
    let css = String::from_utf8(scss.stdout).unwrap();
    
    let markup = html! {
        xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" {
            xsl:output method="html" indent="yes" {}

            xsl:template match="/" {
                div#__pm__panel {
                    style type="text/css" (css)
                    style type="text/css" "@import url('//cdnjs.cloudflare.com/ajax/libs/highlight.js/9.10.0/styles/agate.min.css');"
                    form#__pm__commit method="post" name="commit" {
                        input type="hidden" id="__pm__commit__task" name="task" value="{/state/task/name}" {}
                        ul#__pm__commits {
                            xsl:apply-templates select="/state/log/commit" {}
                        }
                        hr {}
                        dl.properties {
                            xsl:apply-templates select="/state/properties/property" {}
                        }
                        textarea id="__pm__commit__message" tabindex="1" name="message" placeholder="Add a Comment" {
                            xsl:value-of select="/state/message" {}
                        }
                        div#__pm__new_commit {
                            input type="submit" tabindex="4" name="save_update" value="Save Update" {}
                            xsl:if test="/state/changes/change" {
                                fieldset#__pm__commit__changes.details {
                                    legend#__pm__commit__changes_legend tabindex="2" role="button" {
                                        "Include Changes"
                                        span#__pm__commit__changes__statistics.token {
                                            xsl:if test="/state/changes/statistics/files != 0" {
                                                span {
                                                    xsl:value-of select="format-number(/state/changes/statistics/files, '#,###.##')" {}
                                                    " file"
                                                    xsl:if test="/state/changes/statistics/files != 1" {
                                                        "s"
                                                    }
                                                }
                                            }
                                            xsl:if test="/state/changes/statistics/insertions != 0" {
                                                span.token--positive {
                                                    "+"
                                                    xsl:value-of select="format-number(/state/changes/statistics/insertions, '#,###.##')" {}
                                                }
                                            }
                                            xsl:if test="/state/changes/statistics/deletions != 0" {
                                                span.token--negative {
                                                    "-"
                                                    xsl:value-of select="format-number(/state/changes/statistics/deletions, '#,###.##')" {}
                                                }
                                            }
                                        }
                                    }
                                    ul {
                                        xsl:apply-templates select="/state/changes/change" {}
                                    }
                                }
                            }
                        }
                    }
                    xsl:choose {
                        xsl:when test="/state/diffs/*" {
                            xsl:apply-templates select="/state/diffs" {}
                        }
                        xsl:otherwise {
                            xsl:apply-templates select="/state/tasks" {}
                        }
                    }
                }
            }
            xsl:template match="/state/properties/property" {
                div.property {
                    xsl:choose {
                        xsl:when test="name = 'Status'" {
                            dt { label for="__pm__commit__properties--status" "Status" }
                            dd.select tabindex="1" {
                                xsl:value-of select="/state/task/property[name[text()='Status']]/value" {}
                                select id="__pm__commit__properties--status" name="property" data-name="Status" {
                                    xsl:apply-templates "options/option" {}
                                }
                            }
                        }
                        xsl:when test="name = 'Developer'" {
                            dt { label for="__pm__commit__properties--developer" "Developer" }
                            dd.select {
                                input type="text" id="__pm__commit__properties--developer" value="{/state/task/property[name[text()='Developer']]/value}" {}
                                select name="property" data-name="Developer" {
                                    xsl:apply-templates "options/option" {}
                                }
                            }
                        }
                        xsl:when test="name = 'Description'" {
                            dt { label for="__pm__commit__properties--description" "Description" }
                            dd {
                                textarea id="__pm__commit__properties--description" name="property" data-name="Description" {
                                    xsl:value-of select="/state/task/property[name[text()='Description']]/value" {}
                                }
                            }
                        }
                        xsl:otherwise {
                            dt { label for="__pm__commit__properties--estimate" "Estimate" }
                            dd.input {
                                input type="text" id="__pm__commit__properties--estimate" name="property" data-name="Estimate" value="{/state/task/property[name[text()='Estimate']]/value}" {}
                            }
                        }
                    }
                }
            }
            xsl:template match="/state/properties/property/options/option" {
                xsl:variable name="name" select="./parent::options/parent::property/name" {}
                xsl:element name="option" {
                    xsl:attribute name="value" {
                        xsl:value-of select="." {}
                    }
                    xsl:if test="/state/task/property[name[text()=$name]]/value = ." {
                        xsl:attribute name="selected" "selected"
                    }
                    xsl:value-of select="." {}
                }
            }
            xsl:template match="/state/diffs" {
                div.diff {
                    pre {
                        code {
                            xsl:copy-of select="*" {}
                        }
                    }
                }
            }
            xsl:template match="task" {
                li {
                    xsl:element name="div" {
                        xsl:attribute name="draggable" "true"
                        xsl:attribute name="class" {
                            " task "
                            xsl:if test="/state/task/name = name" " selected "
                            xsl:value-of select="concat('task--status-', translate(property[name='Status']/value, 'ABCDEFGHIJKLMNOPQRSTUVWXYZ', 'abcdefghijklmnopqrstuvwxyz'))" {}
                        }
                        xsl:attribute name="data-name" {
                            xsl:value-of select="name" {}
                        }
                        strong {
                            xsl:value-of select="name" {}
                        }
                        xsl:value-of select="property[name='Description']/value" {}
                    }
                }
            }
            xsl:template match="/state/tasks" {
                ul.tiles data-property-name="Status" data-property-value="Sprint" {
                    header "Sprint"
                    xsl:apply-templates select="//task[not(property)]|//task[property[name='Status']/value='Sprint']" {}
                }
                ul.tiles data-property-name="Status" data-property-value="In Progress" {
                    header "In Progress"
                    xsl:apply-templates select="//task[property[name='Status']/value='In Progress']" {}
                }
                ul.tiles data-property-name="Status" data-property-value="In Review" {
                    header "In Review"
                    xsl:apply-templates select="//task[property[name='Status']/value='In Review']" {}
                }
                ul.tiles data-property-name="Status" data-property-value="Blocked" {
                    header "Blocked"
                    xsl:apply-templates select="//task[property[name='Status']/value='Blocked']" {}
                }
                ul.tiles data-property-name="Status" data-property-value="Done" {
                    header "Done"
                    xsl:apply-templates select="//task[property[name='Status']/value='Done']" {}
                }
            }
            xsl:template match="/state/log/commit" {
                li {
                    xsl:if test="./preceding-sibling::commit[1]/user/email=user/email and ./preceding-sibling::commit[1]/timestamp - timestamp > -7200" {
                        xsl:attribute name="class" "continuous"
                    }
                    img src="{user/image}" alt="{user/name} <{user/email}>" {}
                    xsl:if test="changes" {
                        button.button--medium.attachments {
                            svg id="i-paperclip" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32" width="20" height="20" fill="none" stroke="currentcolor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" {
                                path d="M10 9 L10 24 C10 28 13 30 16 30 19 30 22 28 22 24 L22 6 C22 3 20 2 18 2 16 2 14 3 14 6 L14 23 C14 24 15 25 16 25 17 25 18 24 18 23 L18 9" {}
                            }
                        }
                    }
                    blockquote {
                        a.user__name href="#" {
                            xsl:value-of select="user/name" {}
                        }
                        xsl:value-of select="message" {}
                    }
                    xsl:if test="task" {
                        dl.tasks {
                            xsl:apply-templates select="task[property]" {}
                        }
                    }
                    div {
                        time datetime="{localtime}" {
                            xsl:value-of select="localtime" {}
                        }
                    }
                }
            }
            xsl:template match="/state/log/commit/task" {
                dt { xsl:value-of select="name" {} }
                dd {
                    ul.properties {
                        xsl:apply-templates select="property" {}
                    }
                }
            }
            xsl:template match="/state/log/commit/task/property" {
                li.token {
                    span.name { xsl:value-of select="name" {} }
                    xsl:if test="before" {
                        span.before.token--neutral { xsl:value-of select="before" {} }
                    }
                    span.after.token--positive { xsl:value-of select="value" {} }
                }
            }
            xsl:template match="/state/changes/change" {
                li tabindex="3" id="{concat('__pm__changes__checkbox--', @id)}" {
                    xsl:element name="input" {
                        xsl:attribute name="name" { "include" }
                        xsl:attribute name="id" {
                            xsl:value-of select="concat('__pm__changes--', @id)" {}
                        }
                        xsl:attribute name="value" {
                            xsl:value-of select="path" {}
                        }
                        xsl:attribute name="tabindex" "-1"
                        xsl:attribute name="type" "checkbox"
                        xsl:if test="included='true'" {
                            xsl:attribute name="checked" {}
                        }
                        xsl:if test="removal='true'" {
                            xsl:attribute name="class" { "delete" }
                        }
                    }
                    label for="__pm__changes--{@id}" {
                        xsl:value-of select="path" {}
                    }
                    button.button--tiny name="diff" value="{path}" { " +10 -10" }
                }
            }
        }
    };
    format!("{}{}", XML, markup.into_string())
}


