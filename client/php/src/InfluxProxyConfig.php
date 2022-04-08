<?php

declare(strict_types=1);

namespace Easybill\Influxproxy;

class InfluxProxyConfig
{
    public function __construct(
        private $bucket,
        private $org,
        private $endpoint = 'http://127.0.0.1:3343',
    )
    {
    }

    public function getEndpoint(): string
    {
        return $this->endpoint;
    }

    public function getBucket()
    {
        return $this->bucket;
    }

    public function getOrg()
    {
        return $this->org;
    }
}