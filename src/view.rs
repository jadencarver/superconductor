use std::process::Command;
use XML;

pub fn panel_xslt() -> String {
    let scss = Command::new("/usr/local/bin/sassc").arg("/Users/jadencarver/dev/superconductor/assets/__pm.scss").output().unwrap();
    let css = String::from_utf8(scss.stdout).unwrap();
    
    let markup = html! {
        xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" {
            xsl:output method="html" indent="yes" {}

            // BEGINS FORM  ------------------
            xsl:template match="/" {
                div#__pm__panel {
                    form#__pm__commit method="post" name="commit" {
                        style type="text/css" (css)
                        style type="text/css" "@import url('//cdnjs.cloudflare.com/ajax/libs/highlight.js/9.10.0/styles/agate.min.css');"
                        div#__pm__task {
                            input type="checkbox" id="__pm__commit__dragged" name="dragged" value="true" {}
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
                                ul#__pm__new_commit__actions {
                                    li input type="submit" tabindex="5" name="new_task" value="New Task" {}
                                    li input type="submit" tabindex="4" name="save_update" value="Save Update" {}
                                }
                                xsl:if test="/state/changes/change" {
                                    fieldset#__pm__commit__changes.details {
                                        legend#__pm__commit__changes_legend tabindex="2" role="button" {
                                            xsl:if test="not(/state/changes/statistics)" {
                                                "Include Changes"
                                            }
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
                            xsl:when test="/state/setup" {
                                div.setup {
                                    h1 "Superconductor"
                                }
                            }
                            xsl:when test="/state/diffs/*" {
                                xsl:apply-templates select="/state/diffs" {}
                            }
                            xsl:otherwise {
                                xsl:apply-templates select="/state/tasks" {}
                            }
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
                        xsl:when test="name = 'Manager'" {
                            dt { label for="__pm__commit__properties--owner" "Manager" }
                            dd.select {
                                input type="text" id="__pm__commit__properties--manager" value="{/state/task/property[name[text()='Manager']]/value}" {}
                                select name="property" data-name="Manager" {
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
                        xsl:when test="name = 'Estimate'" {
                            dt { label for="__pm__commit__properties--estimate" "Estimate" }
                            dd.input {
                                input type="range" id="__pm__commit__properties--estimate" name="property" data-name="Estimate" value="{/state/task/property[name[text()='Estimate']]/value}" {}
                            }
                        }
                        xsl:otherwise {
                            xsl:variable name="name" select="name/text()" {}
                            xsl:variable name="value" select="/state/task/property[name[text() = $name]]/value" {}
                            input type="hidden" name="property" data-name="{name}" value="{$value}" {}
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

            // BEGINS TASKS  ------------------
            xsl:key name="task-status" match="/state/task|/state/tasks/task" use="property[name='Status']/value" {}

            xsl:template match="/state/tasks" {
                xsl:choose {
                    xsl:when test="filter" {
                        input type="hidden" name="filter" data-name="{filter/name}" data-value="{filter/value}" {}
                        ul.list {
                            header {
                                button type="submit" name="filter" {
                                    xsl:value-of select="filter/value" {}
                                }
                            }
                            xsl:apply-templates select="/state/task|./task" {
                                xsl:sort select="property[name='Ordinal']/value" {}
                            }
                        }
                    }
                    xsl:otherwise {
                        xsl:for-each select="/state/properties/property[name='Status']/options/option" {
                            xsl:variable name="status" select="./text()" {}
                            xsl:variable name="max" {
                                xsl:for-each select="key('task-status', $status)/property[name='Ordinal']/value" {
                                    xsl:sort select="." data-type="number" order="descending" {}
                                        xsl:if test="position() = 1" {
                                            xsl:value-of select="." {}
                                        }
                                }
                            }
                            xsl:variable name="next" {
                                xsl:choose {
                                    xsl:when test="not($max) or string(number($max)) = 'NaN'" { "1" }
                                    xsl:otherwise {
                                        xsl:value-of select="$max + 1" {}
                                    }
                                }
                            }
                            ul.tiles data-property-name="Status" data-property-value="{.}" {
                                div.column data-property-name="Ordinal" data-property-value="{$next}" {
                                    header {
                                        button type="submit" name="filter" data-name="Status" data-value="{.}" {
                                            xsl:value-of select="." {}
                                        }
                                    }
                                    xsl:for-each select="key('task-status', $status)" {
                                        xsl:sort select="property[name='Ordinal']/value" data-type="number" {}
                                        xsl:call-template name="task" {}
                                    }
                                }
                            }
                        }
                    }
                }
            }
            xsl:template match="task" name="task" {
                xsl:variable name="ordinal" {
                    xsl:choose {
                        xsl:when test="property[name='Ordinal']/value != ''" {
                            xsl:value-of select="property[name='Ordinal']/value" {}
                        }
                        xsl:otherwise {
                            "1"
                        }
                    }
                }
                xsl:variable name="previous" {
                    xsl:for-each select="key('task-status', property[name='Status']/value)/property[name='Ordinal'][number(value) < $ordinal]/value" {
                        xsl:sort select="." data-type="number" order="descending" {}
                        xsl:if test="position() = 1" {
                            xsl:value-of select="." {}
                        }
                    }
                }
                xsl:variable name="next" {
                    xsl:choose {
                        xsl:when test="not($previous) or $previous = ''" {
                            xsl:value-of select="$ordinal div 2" {}
                        }
                        xsl:otherwise {
                            xsl:value-of select="$previous+(number(format-number($ordinal - $previous, '###0.0###;#')) div 2)" {}
                        }
                    }
                }
                xsl:variable name="class" {
                    "tile "
                    xsl:value-of select="concat('tile--status-', translate(property[name='Status']/value, 'ABCDEFGHIJKLMNOPQRSTUVWXYZ ', 'abcdefghijklmnopqrstuvwxyz-'))" {}
                }
                li class="{$class}" data-property-name="Ordinal" data-property-value="{$next}" {
                    xsl:element name="div" {
                        xsl:attribute name="draggable" "true"
                        xsl:attribute name="tabindex" "99"
                        xsl:attribute name="id" {
                            xsl:value-of select="concat('__pm__task_', name)" {}
                        }
                        xsl:attribute name="class" {
                            " task "
                            xsl:if test="/state/task/name = name" { " selected " }
                        }
                        xsl:attribute name="data-name" {
                            xsl:value-of select="name" {}
                        }
                        div class="task--name" {
                            xsl:value-of select="name" {}
                        }
                        div.task__property--description {
                            xsl:copy-of select="property[name='Description']/value" {}
                        }
                        xsl:if test="property[name='Estimate']/value != ''" {
                            div.task__property--estimate {
                                xsl:copy-of select="property[name='Estimate']/value" {}
                            }
                        }
                    }
                }
            }


            // BEGINS LOGS  ------------------
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


