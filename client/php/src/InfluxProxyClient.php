<?php

declare(strict_types=1);

namespace Easybill\Influxproxy;

use Easybill\Influxproxy\Dto\InfluxProxyPoint;
use Easybill\Influxproxy\Exception\InfluxProxySendException;

class InfluxProxyClient
{
    /** @throws InfluxProxySendException */
    public static function sendPoint(InfluxProxyConfig $config, InfluxProxyPoint $influxProxyPoint): void
    {
        self::sendRaw($config, $influxProxyPoint->toLineProtocol());
    }

    /**
     * @param InfluxProxyPoint[] $influxProxyPoints
     *
     * @throws InfluxProxySendException
     */
    public static function sendPoints(InfluxProxyConfig $config, array $influxProxyPoints): void
    {
        self::sendRaw(
            $config,
            implode("\n", array_map(fn(InfluxProxyPoint $point) => $point->toLineProtocol(), $influxProxyPoints))
        );
    }

    public static function sendRaw(InfluxProxyConfig $config, string $data): void
    {
        $ch = curl_init(rtrim($config->getEndpoint(), '/') . '/write/' . $config->getOrg() . '/' . $config->getBucket());

        curl_setopt_array($ch, [
            CURLOPT_HEADER => true,
            CURLOPT_RETURNTRANSFER => true,
            CURLOPT_TIMEOUT_MS => 100,
            CURLOPT_CONNECTTIMEOUT => 30,
            CURLOPT_POST => 1,
            CURLOPT_POSTFIELDS => $data,
        ]);

        $response = curl_exec($ch);
        $curl_errno = curl_errno($ch);
        $curl_error = curl_error($ch);
        curl_close($ch);

        if ($curl_errno > 0) {
            throw new InfluxProxySendException('cURL Error (' . $curl_errno . '): ' . $curl_error . 'response: ' . $response);
        }
    }

}