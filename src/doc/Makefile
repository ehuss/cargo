# This Makefile is used to build the Cargo man pages.
#
# The source for the man pages are located in src/doc/man in Asciidoctor
# format. See https://asciidoctor.org/ for more information.
#
# Just run `make` and it will generate the man pages in src/etc/man and the
# HTML pages in src/doc/man/generated.
#
# There are some Asciidoctor extensions, see the file `asciidoc-extensions.rb`
# for the documentation.

MAN_SOURCE = $(sort $(wildcard man/cargo*.adoc))
COMMANDS = $(notdir $(MAN_SOURCE))
HTML = $(patsubst %.adoc,man/generated/%.html,$(COMMANDS))
MAN_LOCATION = ../etc/man
MAN_TXT_LOCATION = man/generated_txt
MAN = $(patsubst %.adoc,$(MAN_LOCATION)/%.1,$(COMMANDS))
MAN_TXT = $(patsubst %.adoc,$(MAN_TXT_LOCATION)/%.txt,$(COMMANDS))
# "reproducible" prevents embedding the date. The date can cause unnecessary
# churn when no other contents have changed.
ASCIIDOCOPTS = -r ./asciidoc-extension.rb -a reproducible
OTHER_DEPS = asciidoc-extension.rb $(filter-out $(MAN_SOURCE),$(sort $(wildcard man/*.adoc)))

all: commands-html man man_txt
commands-html: $(HTML)
man: $(MAN)
man_txt: $(MAN_TXT)

$(HTML): man/generated/%.html : man/%.adoc asciidoc-extension.rb $(OTHER_DEPS)
	asciidoctor $(ASCIIDOCOPTS) -s $< -o $@

$(MAN): $(MAN_LOCATION)/%.1 : man/%.adoc $(OTHER_DEPS)
	asciidoctor $(ASCIIDOCOPTS) -b manpage $< -o $@

$(MAN_TXT): $(MAN_TXT_LOCATION)/%.txt : $(MAN_LOCATION)/%.1
	man $< | col -bx > $@
