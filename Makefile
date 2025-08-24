ifndef VERBOSE
.SILENT:
endif

GITTAG = $(shell git describe --tag --abbrev=0 2>/dev/null | sed -En 's/v(.*)$$/\1/p')
ifeq ($(findstring -, $(GITTAG)), -)
    GITDEV = $(shell git describe --tag 2>/dev/null | sed -En 's/v(.*)-([0-9]+)-g([0-9a-f]+)$$/.dev.\2+\3/p')
else
    GITDEV = $(shell git describe --tag 2>/dev/null | sed -En 's/v(.*)-([0-9]+)-g([0-9a-f]+)$$/-dev.\2+\3/p')
endif
VERSION := "$(GITTAG)$(GITDEV)"

package-all: win-package linux-package

.PHONY: win-package
win-package: win-binary-x86_64
	mkdir fastq-tools || true
	cp target/x86_64-pc-windows-gnu/release/fastq-tools.exe fastq-tools/
	cp README.md fastq-tools/
	cp LICENSE fastq-tools/
	# first try (linux) zip command, then powershell sub command to create ZIP file
	zip fastq-tools-$(VERSION)_win64.zip fastq-tools/* || powershell Compress-ARCHIVE fastq-tools fastq-tools-$(VERSION)_win64.zip
	rm -rf fastq-tools || true

.PHONY: linux-package
linux-package: linux-binary-x86_64
	mkdir fastq-tools || true
	cp target/x86_64-unknown-linux-gnu/release/fastq-tools fastq-tools/
	cp README.md fastq-tools/
	cp LICENSE fastq-tools/
	tar -czvf fastq-tools-$(VERSION)_linux.tar.gz fastq-tools/
	rm -rf fastq-tools || true

binary-all: win-binary-x86_64 linux-binary-x86_64

.PHONY: win-binary-x86_64
win-binary-x86_64:
	cargo build --release --target=x86_64-pc-windows-gnu

.PHONY: linux-binary-x86_64
linux-binary-x86_64:
	cargo build --release --target=x86_64-unknown-linux-gnu

.PHONY: install
install:
	cargo install --path .

.PHONY: clean
clean:
	cargo clean
	rm -rf fastq-tools 2>/dev/null || true
	rm *_win64.zip 2>/dev/null || true
	rm *_linux.tar.gz 2>/dev/null || true