<?php

declare(strict_types=1);

namespace Easybill\Influxproxy\Dto;

class InfluxProxyPoint
{
    public function __construct(
        private string $measurement,
        private array $tag_set,
        private array $field_set,
        private string $timestamp,
    ) {
        if (empty($field_set)) {
            throw new \LogicException('fieldset must not be empty');
        }

        if (!is_numeric($timestamp)) {
            throw new \LogicException('timestamp must be numeric');
        }
    }

    private static function escape(mixed $v): mixed
    {
        if (!is_string($v)) {
            return $v;
        }

        return addcslashes($v, ',= "\\');
    }

    /**
     * @see https://docs.influxdata.com/influxdb/v1.8/write_protocols/line_protocol_tutorial/#syntax
     */
    public function toLineProtocol(): string
    {
        $tags = '';
        foreach ($this->field_set as $k => $v) {
            $tags .= ',' . self::escape($k) . '=' . self::escape($v);
        }
        $fields = '';
        foreach ($this->tag_set as $k => $v) {
            $fields .= ',' . self::escape($k) . '=' . self::escape($v);
        }

        return self::escape($this->measurement) . $tags . ' ' . ltrim($fields, ',') . ' ' . self::escape($this->timestamp);
    }
}