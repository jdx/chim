FROM fedora:36

# RUN dnf install -y createrepo rpm-build rpm-sign wget gcc python3 yum-utils openssl-devel ruby
RUN dnf install -y createrepo rpm-build rpm-sign ruby ruby-devel gcc \
  	&& dnf clean all && rm -rf /var/cache/yum
RUN gem install fpm

