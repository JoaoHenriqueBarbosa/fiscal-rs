FROM php:8.3-cli

RUN apt-get update && apt-get install -y \
    libxml2-dev libssl-dev libzip-dev unzip git \
    && rm -rf /var/lib/apt/lists/* \
    && docker-php-ext-install soap

COPY --from=composer:2 /usr/bin/composer /usr/bin/composer

WORKDIR /app
RUN git clone --depth 1 -b benchmarks https://github.com/JoaoHenriqueBarbosa/sped-nfe.git .
RUN composer install --no-dev --no-interaction --prefer-dist --optimize-autoloader

CMD ["php", "benchmarks/bench_runner.php", "--json"]
